use arqonhpo_core::artifact::SeedPoint;
use arqonhpo_core::config::SolverConfig;
use arqonhpo_core::machine::Solver;
use clap::{Parser, Subcommand};
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "arqonhpo", version, about = "ArqonHPO CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        script: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
    },
    Ask {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
        #[arg(long)]
        batch: Option<usize>,
    },
    Tell {
        #[arg(long)]
        state: PathBuf,
        #[arg(long)]
        results: Option<PathBuf>,
    },
    Interactive {
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        state: Option<PathBuf>,
    },
    Validate {
        #[arg(long)]
        config: PathBuf,
    },
}

#[derive(Serialize, Deserialize)]
struct SolverState {
    config: SolverConfig,
    history: Vec<SeedPoint>,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
enum InteractiveCommand {
    Ask { batch: Option<usize> },
    Tell { results: Vec<SeedPoint> },
}

#[derive(Serialize)]
struct InteractiveAskResponse {
    params: Option<Vec<HashMap<String, f64>>>,
}

#[derive(Serialize)]
struct InteractiveTellResponse {
    ok: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            config,
            script,
            state,
        } => run_command(&config, &script, state.as_ref()),
        Commands::Ask {
            config,
            state,
            batch,
        } => ask_command(&config, state.as_ref(), batch),
        Commands::Tell { state, results } => tell_command(&state, results.as_ref()),
        Commands::Interactive { config, state } => interactive_command(&config, state.as_ref()),
        Commands::Validate { config } => validate_command(&config),
    }
}

fn run_command(config_path: &Path, script: &Path, state_path: Option<&PathBuf>) -> Result<()> {
    let (config, history) = load_state_or_config(config_path, state_path)?;
    let mut solver = Solver::pcr(config.clone());
    if !history.is_empty() {
        solver.seed(history.clone());
    }

    while let Some(candidates) = solver.ask() {
        let mut results = Vec::with_capacity(candidates.len());
        for params in candidates {
            let value = evaluate_script(script, &params)?;
            results.push(SeedPoint {
                params,
                value,
                cost: 1.0,
            });
        }
        solver.seed(results);
    }

    if let Some(path) = state_path {
        let state = SolverState {
            config: solver.config.clone(),
            history: solver
                .history
                .iter()
                .map(|trace| SeedPoint {
                    params: trace.params.clone(),
                    value: trace.value,
                    cost: trace.cost,
                })
                .collect(),
        };
        save_state(path, &state)?;
    }

    let output = serde_json::to_string_pretty(&solver.history).into_diagnostic()?;
    println!("{}", output);
    Ok(())
}

fn ask_command(
    config_path: &Path,
    state_path: Option<&PathBuf>,
    batch: Option<usize>,
) -> Result<()> {
    let (config, history) = load_state_or_config(config_path, state_path)?;
    let mut solver = Solver::pcr(config);
    if !history.is_empty() {
        solver.seed(history);
    }

    let mut response = solver.ask();
    if let (Some(limit), Some(ref mut candidates)) = (batch, response.as_mut()) {
        if candidates.len() > limit {
            candidates.truncate(limit);
        }
    }

    let output = serde_json::to_string_pretty(&response).into_diagnostic()?;
    println!("{}", output);
    Ok(())
}

fn tell_command(state_path: &Path, results_path: Option<&PathBuf>) -> Result<()> {
    let mut state = load_state(state_path)?;
    let results_json = read_input(results_path)?;
    let mut results: Vec<SeedPoint> = serde_json::from_str(&results_json).into_diagnostic()?;
    state.history.append(&mut results);
    save_state(state_path, &state)?;
    Ok(())
}

fn interactive_command(config_path: &Path, state_path: Option<&PathBuf>) -> Result<()> {
    let (config, history) = load_state_or_config(config_path, state_path)?;
    let mut solver = Solver::pcr(config.clone());
    if !history.is_empty() {
        solver.seed(history);
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.into_diagnostic()?;
        if line.trim().is_empty() {
            continue;
        }
        let command: InteractiveCommand = serde_json::from_str(&line).into_diagnostic()?;
        match command {
            InteractiveCommand::Ask { batch } => {
                let mut response = solver.ask();
                if let (Some(limit), Some(ref mut candidates)) = (batch, response.as_mut()) {
                    if candidates.len() > limit {
                        candidates.truncate(limit);
                    }
                }
                let payload = InteractiveAskResponse { params: response };
                writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string(&payload).into_diagnostic()?
                )
                .into_diagnostic()?;
            }
            InteractiveCommand::Tell { results } => {
                solver.seed(results);
                let payload = InteractiveTellResponse { ok: true };
                writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string(&payload).into_diagnostic()?
                )
                .into_diagnostic()?;
                if let Some(path) = state_path {
                    let state = SolverState {
                        config: config.clone(),
                        history: solver
                            .history
                            .iter()
                            .map(|trace| SeedPoint {
                                params: trace.params.clone(),
                                value: trace.value,
                                cost: trace.cost,
                            })
                            .collect(),
                    };
                    save_state(path, &state)?;
                }
            }
        }
        stdout.flush().into_diagnostic()?;
    }
    Ok(())
}

fn validate_command(config_path: &Path) -> Result<()> {
    let config = load_config(config_path)?;
    validate_config(&config)?;
    println!("Config OK");
    Ok(())
}

fn load_state_or_config(
    config_path: &Path,
    state_path: Option<&PathBuf>,
) -> Result<(SolverConfig, Vec<SeedPoint>)> {
    let config = load_config(config_path)?;
    if let Some(path) = state_path {
        if path.exists() {
            let state = load_state(path)?;
            return Ok((state.config, state.history));
        }
    }
    Ok((config, Vec::new()))
}

fn load_state(path: &Path) -> Result<SolverState> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read state file {}", path.display()))?;
    let state: SolverState = serde_json::from_str(&contents)
        .into_diagnostic()
        .with_context(|| format!("Invalid state JSON in {}", path.display()))?;
    Ok(state)
}

fn save_state(path: &Path, state: &SolverState) -> Result<()> {
    let data = serde_json::to_string_pretty(state).into_diagnostic()?;
    fs::write(path, data)
        .into_diagnostic()
        .with_context(|| format!("Failed to write state file {}", path.display()))?;
    Ok(())
}

fn load_config(path: &Path) -> Result<SolverConfig> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read config file {}", path.display()))?;
    let config: SolverConfig = serde_json::from_str(&contents)
        .into_diagnostic()
        .with_context(|| format!("Invalid config JSON in {}", path.display()))?;
    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &SolverConfig) -> Result<()> {
    if config.budget == 0 {
        return Err(miette::miette!("budget must be > 0"));
    }
    if config.bounds.is_empty() {
        return Err(miette::miette!("bounds must not be empty"));
    }
    for (name, domain) in &config.bounds {
        if domain.min >= domain.max {
            return Err(miette::miette!(
                "bounds for {} must satisfy min < max",
                name
            ));
        }
        if matches!(domain.scale, arqonhpo_core::config::Scale::Log)
            && (domain.min <= 0.0 || domain.max <= 0.0)
        {
            return Err(miette::miette!("log scale bounds for {} must be > 0", name));
        }
    }
    Ok(())
}

fn read_input(path: Option<&PathBuf>) -> Result<String> {
    if let Some(path) = path {
        return fs::read_to_string(path)
            .into_diagnostic()
            .with_context(|| format!("Failed to read results file {}", path.display()));
    }

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).into_diagnostic()?;
    Ok(buffer)
}

fn evaluate_script(script: &Path, params: &HashMap<String, f64>) -> Result<f64> {
    let mut command = Command::new(script);
    let mut keys: Vec<_> = params.keys().collect();
    keys.sort();
    for key in keys {
        let env_key = format!("ARQON_{}", key);
        command.env(env_key, params[key].to_string());
    }

    let output = command.output().into_diagnostic()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(miette::miette!(
            "Script failed with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_result(&stdout)
}

fn parse_result(stdout: &str) -> Result<f64> {
    let mut last_value: Option<&str> = None;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("RESULT=") {
            last_value = Some(rest.trim());
        } else {
            last_value = Some(trimmed);
        }
    }

    let value = last_value.ok_or_else(|| miette::miette!("No RESULT found in script output"))?;
    value
        .parse::<f64>()
        .into_diagnostic()
        .with_context(|| format!("Failed to parse result '{}'", value))
}
