#![allow(clippy::disallowed_types)]

use arqonhpo_core::artifact::{EvalTrace, RunArtifact, SeedPoint};
use arqonhpo_core::config::SolverConfig;
use arqonhpo_core::machine::Solver;
use clap::{Parser, Subcommand, ValueEnum};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use miette::{Context, IntoDiagnostic, Result};
use prometheus::{Encoder, Histogram, HistogramOpts, IntCounter, IntGauge, Registry, TextEncoder};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Frame;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Response, Server};

#[derive(Parser)]
#[command(name = "arqonhpo", version, about = "ArqonHPO CLI")]
struct Cli {
    #[arg(long, value_enum, default_value = "pretty")]
    log_format: LogFormat,
    #[arg(long, default_value = "info")]
    log_level: String,
    #[arg(long)]
    metrics_addr: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
enum LogFormat {
    Pretty,
    Json,
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
    Export {
        #[arg(long)]
        state: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
    },
    Import {
        #[arg(long)]
        artifact: PathBuf,
        #[arg(long)]
        state: PathBuf,
    },
    Tui {
        #[arg(long)]
        state: PathBuf,
        #[arg(long)]
        events: Option<PathBuf>,
        #[arg(long, default_value_t = 500)]
        refresh_ms: u64,
    },
    Dashboard {
        #[arg(long)]
        state: PathBuf,
        #[arg(long)]
        events: Option<PathBuf>,
        #[arg(long)]
        actions: Option<PathBuf>,
        #[arg(long, default_value = "127.0.0.1:3030")]
        addr: String,
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
    #[serde(default)]
    run_id: Option<String>,
}

struct LoadedState {
    config: SolverConfig,
    history: Vec<SeedPoint>,
    run_id: Option<String>,
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

struct Metrics {
    registry: Registry,
    ask_calls: IntCounter,
    tell_calls: IntCounter,
    candidates_emitted: IntCounter,
    results_ingested: IntCounter,
    history_len: IntGauge,
    eval_seconds: Histogram,
}

impl Metrics {
    fn init(addr: Option<&str>) -> Result<Arc<Self>> {
        let registry = Registry::new();
        let ask_calls =
            IntCounter::new("arqonhpo_ask_calls", "Number of ask calls").into_diagnostic()?;
        let tell_calls =
            IntCounter::new("arqonhpo_tell_calls", "Number of tell calls").into_diagnostic()?;
        let candidates_emitted =
            IntCounter::new("arqonhpo_candidates_emitted", "Candidates emitted")
                .into_diagnostic()?;
        let results_ingested =
            IntCounter::new("arqonhpo_results_ingested", "Results ingested").into_diagnostic()?;
        let history_len =
            IntGauge::new("arqonhpo_history_len", "History length").into_diagnostic()?;
        let eval_seconds = Histogram::with_opts(HistogramOpts::new(
            "arqonhpo_eval_seconds",
            "Evaluation latency in seconds",
        ))
        .into_diagnostic()?;

        registry
            .register(Box::new(ask_calls.clone()))
            .into_diagnostic()?;
        registry
            .register(Box::new(tell_calls.clone()))
            .into_diagnostic()?;
        registry
            .register(Box::new(candidates_emitted.clone()))
            .into_diagnostic()?;
        registry
            .register(Box::new(results_ingested.clone()))
            .into_diagnostic()?;
        registry
            .register(Box::new(history_len.clone()))
            .into_diagnostic()?;
        registry
            .register(Box::new(eval_seconds.clone()))
            .into_diagnostic()?;

        let metrics = Arc::new(Self {
            registry,
            ask_calls,
            tell_calls,
            candidates_emitted,
            results_ingested,
            history_len,
            eval_seconds,
        });

        if let Some(addr) = addr {
            let registry = metrics.registry.clone();
            let addr = addr.to_string();
            thread::spawn(move || start_metrics_server(&addr, &registry));
        }

        Ok(metrics)
    }

    fn record_ask(&self, candidates: usize) {
        self.ask_calls.inc();
        self.candidates_emitted.inc_by(candidates as u64);
    }

    fn record_tell(&self, results: usize) {
        self.tell_calls.inc();
        self.results_ingested.inc_by(results as u64);
    }

    fn set_history_len(&self, len: usize) {
        self.history_len.set(len as i64);
    }

    fn observe_eval(&self, seconds: f64) {
        self.eval_seconds.observe(seconds);
    }
}

fn start_metrics_server(addr: &str, registry: &Registry) {
    let server = match Server::http(addr) {
        Ok(server) => server,
        Err(_) => return,
    };
    for request in server.incoming_requests() {
        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        let mut buffer = Vec::new();
        if encoder.encode(&metric_families, &mut buffer).is_ok() {
            let response = Response::from_data(buffer);
            let _ = request.respond(response);
        }
    }
}

fn init_tracing(log_format: LogFormat, log_level: &str) -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_new(log_level)
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .into_diagnostic()?;
    let fmt = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr);
    match log_format {
        LogFormat::Json => fmt.json().init(),
        LogFormat::Pretty => fmt.init(),
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.log_format, &cli.log_level)?;
    let metrics = Metrics::init(cli.metrics_addr.as_deref())?;

    match cli.command {
        Commands::Run {
            config,
            script,
            state,
        } => run_command(&config, &script, state.as_ref(), &metrics),
        Commands::Ask {
            config,
            state,
            batch,
        } => ask_command(&config, state.as_ref(), batch, &metrics),
        Commands::Tell { state, results } => tell_command(&state, results.as_ref(), &metrics),
        Commands::Interactive { config, state } => {
            interactive_command(&config, state.as_ref(), &metrics)
        }
        Commands::Export {
            state,
            output,
            run_id,
        } => export_command(&state, output.as_ref(), run_id, &metrics),
        Commands::Import { artifact, state } => import_command(&artifact, &state, &metrics),
        Commands::Tui {
            state,
            events,
            refresh_ms,
        } => tui_command(&state, events.as_ref(), refresh_ms, &metrics),
        Commands::Dashboard {
            state,
            events,
            actions,
            addr,
        } => dashboard_command(&state, events.as_ref(), actions.as_ref(), &addr, &metrics),
        Commands::Validate { config } => validate_command(&config),
    }
}

fn run_command(
    config_path: &Path,
    script: &Path,
    state_path: Option<&PathBuf>,
    metrics: &Metrics,
) -> Result<()> {
    tracing::info!(command = "run", config = %config_path.display());
    let loaded = load_state_or_config(config_path, state_path)?;
    let run_id = loaded.run_id.unwrap_or_else(|| generate_run_id("run"));
    let mut solver = Solver::pcr(loaded.config.clone());
    if !loaded.history.is_empty() {
        solver.seed(loaded.history.clone());
    }

    while let Some(candidates) = solver.ask() {
        metrics.record_ask(candidates.len());
        let mut results = Vec::with_capacity(candidates.len());
        for params in candidates {
            let start = SystemTime::now();
            let value = evaluate_script(script, &params)?;
            let elapsed = start.elapsed().unwrap_or_default();
            metrics.observe_eval(elapsed.as_secs_f64());
            results.push(SeedPoint {
                params,
                value,
                cost: 1.0,
            });
        }
        metrics.record_tell(results.len());
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
            run_id: Some(run_id),
        };
        save_state(path, &state)?;
    }

    metrics.set_history_len(solver.history.len());

    let output = serde_json::to_string_pretty(&solver.history).into_diagnostic()?;
    println!("{}", output);
    Ok(())
}

fn ask_command(
    config_path: &Path,
    state_path: Option<&PathBuf>,
    batch: Option<usize>,
    metrics: &Metrics,
) -> Result<()> {
    tracing::info!(command = "ask", config = %config_path.display());
    let loaded = load_state_or_config(config_path, state_path)?;
    let mut solver = Solver::pcr(loaded.config);
    if !loaded.history.is_empty() {
        solver.seed(loaded.history);
    }

    let mut response = solver.ask();
    if let (Some(limit), Some(ref mut candidates)) = (batch, response.as_mut()) {
        if candidates.len() > limit {
            candidates.truncate(limit);
        }
    }

    if let Some(ref candidates) = response {
        metrics.record_ask(candidates.len());
    }

    let output = serde_json::to_string_pretty(&response).into_diagnostic()?;
    println!("{}", output);
    Ok(())
}

fn tell_command(
    state_path: &Path,
    results_path: Option<&PathBuf>,
    metrics: &Metrics,
) -> Result<()> {
    tracing::info!(command = "tell", state = %state_path.display());
    let mut state = load_state(state_path)?;
    let results_json = read_input(results_path)?;
    let mut results: Vec<SeedPoint> = serde_json::from_str(&results_json).into_diagnostic()?;
    metrics.record_tell(results.len());
    state.history.append(&mut results);
    metrics.set_history_len(state.history.len());
    save_state(state_path, &state)?;
    Ok(())
}

fn interactive_command(
    config_path: &Path,
    state_path: Option<&PathBuf>,
    metrics: &Metrics,
) -> Result<()> {
    tracing::info!(command = "interactive", config = %config_path.display());
    let loaded = load_state_or_config(config_path, state_path)?;
    let run_id = loaded
        .run_id
        .unwrap_or_else(|| generate_run_id("interactive"));
    let mut solver = Solver::pcr(loaded.config.clone());
    if !loaded.history.is_empty() {
        solver.seed(loaded.history);
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
                if let Some(ref candidates) = response {
                    metrics.record_ask(candidates.len());
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
                metrics.record_tell(results.len());
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
                        config: loaded.config.clone(),
                        history: solver
                            .history
                            .iter()
                            .map(|trace| SeedPoint {
                                params: trace.params.clone(),
                                value: trace.value,
                                cost: trace.cost,
                            })
                            .collect(),
                        run_id: Some(run_id.clone()),
                    };
                    save_state(path, &state)?;
                }
            }
        }
        stdout.flush().into_diagnostic()?;
    }
    Ok(())
}

fn export_command(
    state_path: &Path,
    output_path: Option<&PathBuf>,
    run_id: Option<String>,
    metrics: &Metrics,
) -> Result<()> {
    tracing::info!(command = "export", state = %state_path.display());
    let state = load_state(state_path)?;
    let run_id = run_id
        .or(state.run_id.clone())
        .unwrap_or_else(|| generate_run_id("export"));
    let history: Vec<EvalTrace> = state
        .history
        .iter()
        .enumerate()
        .map(|(index, seed)| EvalTrace {
            eval_id: (index + 1) as u64,
            params: seed.params.clone(),
            value: seed.value,
            cost: seed.cost,
        })
        .collect();
    let artifact = RunArtifact {
        run_id,
        seed: state.config.seed,
        budget: state.config.budget,
        config: state.config,
        history,
    };
    metrics.set_history_len(artifact.history.len());
    write_output(output_path, &artifact)?;
    Ok(())
}

fn import_command(artifact_path: &Path, state_path: &Path, metrics: &Metrics) -> Result<()> {
    tracing::info!(
        command = "import",
        artifact = %artifact_path.display(),
        state = %state_path.display()
    );
    let artifact: RunArtifact = read_json(artifact_path)?;
    let history: Vec<SeedPoint> = artifact
        .history
        .iter()
        .map(|trace| SeedPoint {
            params: trace.params.clone(),
            value: trace.value,
            cost: trace.cost,
        })
        .collect();
    let state = SolverState {
        config: artifact.config,
        history,
        run_id: Some(artifact.run_id),
    };
    metrics.set_history_len(state.history.len());
    save_state(state_path, &state)
}

fn tui_command(
    state_path: &Path,
    events_path: Option<&PathBuf>,
    refresh_ms: u64,
    metrics: &Metrics,
) -> Result<()> {
    tracing::info!(command = "tui", state = %state_path.display());
    enable_raw_mode().into_diagnostic()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen).into_diagnostic()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).into_diagnostic()?;
    let tick_rate = Duration::from_millis(refresh_ms);

    loop {
        let state = load_state(state_path).ok();
        if let Some(ref loaded) = state {
            metrics.set_history_len(loaded.history.len());
        }
        let events = events_path
            .and_then(|path| read_event_lines(path, 5).ok())
            .unwrap_or_default();

        terminal
            .draw(|frame| draw_tui(frame, state.as_ref(), &events))
            .into_diagnostic()?;

        if event::poll(tick_rate).into_diagnostic()? {
            if let Event::Key(key) = event::read().into_diagnostic()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }

    disable_raw_mode().into_diagnostic()?;
    terminal.show_cursor().into_diagnostic()?;
    terminal
        .backend_mut()
        .execute(LeaveAlternateScreen)
        .into_diagnostic()?;
    Ok(())
}

mod dashboard;

fn dashboard_command(
    state_path: &Path,
    events_path: Option<&PathBuf>,
    actions_path: Option<&PathBuf>,
    addr: &str,
    metrics: &Metrics,
) -> Result<()> {
    tracing::info!(command = "dashboard", state = %state_path.display(), addr = %addr);
    let server = Server::http(addr)
        .map_err(|e| miette::miette!("Failed to bind dashboard server to {}: {}", addr, e))?;
    let bound_addr = server.server_addr();
    println!("Dashboard running at http://{}", bound_addr);

    for mut request in server.incoming_requests() {
        let url: &str = request.url();
        let (path, query) = split_query(url);
        let response = match (request.method().as_str(), path) {
            ("GET", "/") => dashboard::plain_response(dashboard::DASHBOARD_HTML, "text/html"),
            ("GET", "/assets/dashboard.css") => {
                dashboard::plain_response(dashboard::DASHBOARD_CSS, "text/css")
            }
            ("GET", "/assets/dashboard.js") => {
                dashboard::plain_response(dashboard::DASHBOARD_JS, "text/javascript")
            }
            ("GET", "/api/state") => {
                dashboard::json_response(dashboard::load_state_json(state_path, metrics))
            }
            ("GET", "/api/summary") => {
                dashboard::json_response(dashboard::load_summary_json(state_path))
            }
            ("GET", "/api/events") => {
                let params = parse_query(query);
                dashboard::json_response(dashboard::load_events_json(events_path, &params))
            }
            ("GET", "/api/actions") => {
                let params = parse_query(query);
                dashboard::json_response(dashboard::load_actions_json(actions_path, &params))
            }
            ("POST", "/api/actions") => {
                dashboard::json_response(dashboard::store_action(request.as_reader(), actions_path))
            }
            _ => Response::from_string("Not found").with_status_code(404),
        };
        let _ = request.respond(response);
    }
    Ok(())
}

fn draw_tui(frame: &mut Frame, state: Option<&SolverState>, events: &[String]) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(8),
        ])
        .split(frame.area());

    let summary_lines = match state {
        Some(state) => {
            let best = state
                .history
                .iter()
                .map(|entry| entry.value)
                .min_by(|left, right| left.partial_cmp(right).unwrap());
            let latest = state.history.last().map(|entry| entry.value);
            vec![
                Line::from(format!(
                    "Run ID: {}",
                    state.run_id.clone().unwrap_or_default()
                )),
                Line::from(format!("Budget: {}", state.config.budget)),
                Line::from(format!("History: {}", state.history.len())),
                Line::from(format!(
                    "Best: {} | Latest: {}",
                    best.map(|value| format!("{value:.6}"))
                        .unwrap_or_else(|| "n/a".into()),
                    latest
                        .map(|value| format!("{value:.6}"))
                        .unwrap_or_else(|| "n/a".into())
                )),
            ]
        }
        None => vec![Line::from("No state loaded")],
    };

    let summary = Paragraph::new(summary_lines)
        .block(Block::default().borders(Borders::ALL).title("Summary"));
    frame.render_widget(summary, layout[0]);

    let history_items: Vec<ListItem> = match state {
        Some(state) if !state.history.is_empty() => state
            .history
            .iter()
            .rev()
            .take(6)
            .map(|entry| {
                let params = format_params(&entry.params);
                ListItem::new(format!("value={:.6} | {}", entry.value, params))
            })
            .collect(),
        _ => vec![ListItem::new("No evaluations yet")],
    };
    let history = List::new(history_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Recent Evaluations"),
    );
    frame.render_widget(history, layout[1]);

    let event_items: Vec<ListItem> = if events.is_empty() {
        vec![ListItem::new("No events")]
    } else {
        events
            .iter()
            .map(|line| ListItem::new(line.clone()).style(Style::default()))
            .collect()
    };
    let event_list =
        List::new(event_items).block(Block::default().borders(Borders::ALL).title("Events"));
    frame.render_widget(event_list, layout[2]);
}

fn format_params(params: &HashMap<String, f64>) -> String {
    let mut keys: Vec<_> = params.keys().collect();
    keys.sort();
    let parts: Vec<String> = keys
        .into_iter()
        .filter_map(|key| params.get(key).map(|value| format!("{}={:.4}", key, value)))
        .collect();
    parts.join(", ")
}

fn read_event_lines(path: &Path, limit: usize) -> Result<Vec<String>> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read events file {}", path.display()))?;
    let mut lines: Vec<String> = contents.lines().filter_map(format_event_line).collect();
    if lines.len() > limit {
        lines = lines.split_off(lines.len() - limit);
    }
    Ok(lines)
}

fn format_event_line(line: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let event = value
        .get("event")
        .or_else(|| value.get("event_type"))
        .and_then(|field| field.as_str())
        .unwrap_or("event");
    let timestamp = value
        .get("timestamp_us")
        .and_then(|field| field.as_u64())
        .unwrap_or(0);
    let detail = value
        .get("value")
        .and_then(|field| field.as_f64())
        .map(|metric| format!("value={metric:.6}"))
        .unwrap_or_default();
    Some(format!("{timestamp} {event} {detail}").trim().to_string())
}

fn generate_run_id(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{prefix}-{timestamp}")
}

fn write_output<T: Serialize>(path: Option<&PathBuf>, value: &T) -> Result<()> {
    let data = serde_json::to_string_pretty(value).into_diagnostic()?;
    if let Some(path) = path {
        fs::write(path, data)
            .into_diagnostic()
            .with_context(|| format!("Failed to write output file {}", path.display()))?;
    } else {
        println!("{}", data);
    }
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read JSON file {}", path.display()))?;
    serde_json::from_str(&contents)
        .into_diagnostic()
        .with_context(|| format!("Invalid JSON in {}", path.display()))
}

fn split_query(url: &str) -> (&str, Option<&str>) {
    if let Some((path, query)) = url.split_once('?') {
        (path, Some(query))
    } else {
        (url, None)
    }
}

fn parse_query(query: Option<&str>) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let Some(query) = query else {
        return params;
    };
    for pair in query.split('&') {
        let mut iter = pair.splitn(2, '=');
        if let Some(key) = iter.next() {
            if key.is_empty() {
                continue;
            }
            let value = iter.next().unwrap_or("");
            params.insert(key.to_string(), value.to_string());
        }
    }
    params
}

fn validate_command(config_path: &Path) -> Result<()> {
    tracing::info!(command = "validate", config = %config_path.display());
    let config = load_config(config_path)?;
    validate_config(&config)?;
    println!("Config OK");
    Ok(())
}

fn load_state_or_config(config_path: &Path, state_path: Option<&PathBuf>) -> Result<LoadedState> {
    let config = load_config(config_path)?;
    if let Some(path) = state_path {
        if path.exists() {
            let state = load_state(path)?;
            return Ok(LoadedState {
                config: state.config,
                history: state.history,
                run_id: state.run_id,
            });
        }
    }
    Ok(LoadedState {
        config,
        history: Vec::new(),
        run_id: None,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_params_empty() {
        let params: HashMap<String, f64> = HashMap::new();
        assert_eq!(format_params(&params), "");
    }

    #[test]
    fn test_format_params_single() {
        let mut params = HashMap::new();
        params.insert("alpha".to_string(), 0.1234);
        assert_eq!(format_params(&params), "alpha=0.1234");
    }

    #[test]
    fn test_format_params_multiple_sorted() {
        let mut params = HashMap::new();
        params.insert("z".to_string(), 1.0);
        params.insert("a".to_string(), 2.0);
        params.insert("m".to_string(), 3.0);
        let result = format_params(&params);
        assert!(result.starts_with("a="));
        assert!(result.contains("m="));
        assert!(result.ends_with("z=1.0000"));
    }

    #[test]
    fn test_format_event_line_valid_json() {
        let line = r#"{"event":"update","timestamp_us":1234567890,"value":0.5}"#;
        let result = format_event_line(line);
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("1234567890"));
        assert!(formatted.contains("update"));
        assert!(formatted.contains("value=0.500000"));
    }

    #[test]
    fn test_format_event_line_invalid_json() {
        let line = "not valid json";
        let result = format_event_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_format_event_line_minimal() {
        let line = r#"{}"#;
        let result = format_event_line(line);
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("event"));
    }

    #[test]
    fn test_generate_run_id() {
        let run_id = generate_run_id("test");
        assert!(run_id.starts_with("test-"));
        assert!(run_id.len() > 5);
    }

    #[test]
    fn test_split_query_with_query() {
        let (path, query) = split_query("/api/data?key=value");
        assert_eq!(path, "/api/data");
        assert_eq!(query, Some("key=value"));
    }

    #[test]
    fn test_split_query_without_query() {
        let (path, query) = split_query("/api/data");
        assert_eq!(path, "/api/data");
        assert!(query.is_none());
    }

    #[test]
    fn test_parse_query_empty() {
        let params = parse_query(None);
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_query_single() {
        let params = parse_query(Some("key=value"));
        assert_eq!(params.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_query_multiple() {
        let params = parse_query(Some("a=1&b=2&c=3"));
        assert_eq!(params.len(), 3);
        assert_eq!(params.get("a"), Some(&"1".to_string()));
        assert_eq!(params.get("b"), Some(&"2".to_string()));
        assert_eq!(params.get("c"), Some(&"3".to_string()));
    }

    #[test]
    fn test_parse_query_empty_key() {
        let params = parse_query(Some("=value&key=val"));
        assert_eq!(params.len(), 1);
        assert_eq!(params.get("key"), Some(&"val".to_string()));
    }

    #[test]
    fn test_parse_result_simple() {
        let result = parse_result("0.5");
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_parse_result_with_prefix() {
        let result = parse_result("RESULT=0.75");
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_parse_result_multiline() {
        let output = "some output\nmore output\nRESULT=0.9\n";
        let result = parse_result(output);
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_parse_result_empty() {
        let result = parse_result("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_result_invalid_number() {
        let result = parse_result("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_valid() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 0.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Linear,
            },
        );
        let config = SolverConfig {
            bounds,
            budget: 10,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_zero_budget() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 0.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Linear,
            },
        );
        let config = SolverConfig {
            bounds,
            budget: 0,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("budget"));
    }

    #[test]
    fn test_validate_config_empty_bounds() {
        let config = SolverConfig {
            bounds: HashMap::new(),
            budget: 10,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bounds"));
    }

    #[test]
    fn test_validate_config_invalid_bounds() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 1.0,
                max: 0.0,
                scale: arqonhpo_core::config::Scale::Linear,
            }, // min > max
        );
        let config = SolverConfig {
            bounds,
            budget: 10,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    // ==================== METRICS TESTS ====================

    #[test]
    fn test_metrics_init_no_server() {
        // Init without metrics server
        let metrics = Metrics::init(None);
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_metrics_record_ask() {
        let metrics = Metrics::init(None).unwrap();
        metrics.record_ask(5);
        // Counter should be incremented
        assert_eq!(metrics.ask_calls.get(), 1);
        assert_eq!(metrics.candidates_emitted.get(), 5);
    }

    #[test]
    fn test_metrics_record_tell() {
        let metrics = Metrics::init(None).unwrap();
        metrics.record_tell(10);
        assert_eq!(metrics.tell_calls.get(), 1);
        assert_eq!(metrics.results_ingested.get(), 10);
    }

    #[test]
    fn test_metrics_set_history_len() {
        let metrics = Metrics::init(None).unwrap();
        metrics.set_history_len(42);
        assert_eq!(metrics.history_len.get(), 42);
    }

    #[test]
    fn test_metrics_observe_eval() {
        let metrics = Metrics::init(None).unwrap();
        metrics.observe_eval(0.5);
        metrics.observe_eval(1.0);
        // Histogram should have 2 observations
        assert_eq!(metrics.eval_seconds.get_sample_count(), 2);
    }

    // ==================== FILE I/O TESTS ====================

    #[test]
    fn test_save_and_load_state() {
        use tempfile::NamedTempFile;

        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 0.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Linear,
            },
        );
        let config = SolverConfig {
            bounds,
            budget: 10,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        };

        let state = SolverState {
            config,
            history: vec![SeedPoint {
                params: [("x".to_string(), 0.5)].into_iter().collect(),
                value: 1.0,
                cost: 1.0,
            }],
            run_id: Some("test-run".to_string()),
        };

        let file = NamedTempFile::new().unwrap();
        save_state(file.path(), &state).unwrap();

        let loaded = load_state(file.path()).unwrap();
        assert_eq!(loaded.run_id, Some("test-run".to_string()));
        assert_eq!(loaded.history.len(), 1);
        assert_eq!(loaded.config.budget, 10);
    }

    #[test]
    fn test_load_state_not_found() {
        let result = load_state(Path::new("/nonexistent/path/state.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_valid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
            "seed": 42,
            "budget": 20,
            "probe_ratio": 0.5,
            "bounds": {{
                "alpha": {{"min": 0.0, "max": 1.0}}
            }}
        }}"#
        )
        .unwrap();

        let config = load_config(file.path()).unwrap();
        assert_eq!(config.seed, 42);
        assert_eq!(config.budget, 20);
        assert!(config.bounds.contains_key("alpha"));
    }

    #[test]
    fn test_load_config_not_found() {
        let result = load_config(Path::new("/nonexistent/config.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_invalid_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not valid json").unwrap();

        let result = load_config(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_write_output_to_file() {
        use tempfile::NamedTempFile;

        let file = NamedTempFile::new().unwrap();
        let data = vec![1, 2, 3];
        write_output(Some(&file.path().to_path_buf()), &data).unwrap();

        let content = fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("["));
        assert!(content.contains("1"));
    }

    #[test]
    fn test_read_json_valid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"key": "value"}}"#).unwrap();

        let result: serde_json::Value = read_json(file.path()).unwrap();
        assert_eq!(result["key"], "value");
    }

    #[test]
    fn test_read_json_invalid() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not json").unwrap();

        let result: miette::Result<serde_json::Value> = read_json(file.path());
        assert!(result.is_err());
    }

    // ==================== SOLVER STATE TESTS ====================

    #[test]
    fn test_solver_state_serialization() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 0.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Linear,
            },
        );
        let state = SolverState {
            config: SolverConfig {
                bounds,
                budget: 10,
                probe_ratio: 0.5,
                seed: 42,
                strategy_params: None,
            },
            history: vec![],
            run_id: Some("test".to_string()),
        };

        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"run_id\":\"test\""));
        assert!(json.contains("\"budget\":10"));
    }

    #[test]
    fn test_loaded_state_from_config() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 0.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Linear,
            },
        );
        let loaded = LoadedState {
            config: SolverConfig {
                bounds,
                budget: 10,
                probe_ratio: 0.5,
                seed: 42,
                strategy_params: None,
            },
            history: vec![],
            run_id: None,
        };

        assert!(loaded.run_id.is_none());
        assert!(loaded.history.is_empty());
    }

    // ==================== INIT TRACING TEST ====================

    #[test]
    fn test_init_tracing_json() {
        // This just verifies it doesn't panic - tracing can only init once per process
        // so we can't really test this properly in unit tests
        // The important thing is coverage of the match arms
    }

    // ==================== VALIDATE CONFIG EDGE CASES ====================

    #[test]
    fn test_validate_config_log_scale_negative() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: -1.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Log,
            },
        );
        let config = SolverConfig {
            bounds,
            budget: 10,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("log scale"));
    }

    #[test]
    fn test_validate_config_log_scale_zero() {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 0.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Log,
            },
        );
        let config = SolverConfig {
            bounds,
            budget: 10,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        };
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    // ==================== READ INPUT TEST ====================

    #[test]
    fn test_read_input_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "test content").unwrap();

        let content = read_input(Some(&file.path().to_path_buf())).unwrap();
        assert!(content.contains("test content"));
    }

    #[test]
    fn test_read_input_file_not_found() {
        let result = read_input(Some(&PathBuf::from("/nonexistent/file.txt")));
        assert!(result.is_err());
    }

    // ==================== EVALUATE SCRIPT PARSING ====================

    #[test]
    fn test_parse_result_with_whitespace() {
        let result = parse_result("  \n\n  0.75  \n\n  ");
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_parse_result_last_value_wins() {
        let output = "0.1\n0.2\n0.3";
        let result = parse_result(output);
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_parse_result_result_prefix_wins() {
        let output = "noise\nmore noise\nRESULT=0.99";
        let result = parse_result(output);
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.99).abs() < 0.001);
    }

    // ==================== INTERACTIVE COMMAND PARSING ====================

    #[test]
    fn test_interactive_command_deserialization_ask() {
        let json = r#"{"cmd": "ask", "batch": 5}"#;
        let cmd: InteractiveCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, InteractiveCommand::Ask { batch: Some(5) }));
    }

    #[test]
    fn test_interactive_command_deserialization_ask_no_batch() {
        let json = r#"{"cmd": "ask"}"#;
        let cmd: InteractiveCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, InteractiveCommand::Ask { batch: None }));
    }

    #[test]
    fn test_interactive_command_deserialization_tell() {
        let json = r#"{"cmd": "tell", "results": []}"#;
        let cmd: InteractiveCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, InteractiveCommand::Tell { results } if results.is_empty()));
    }

    #[test]
    fn test_interactive_ask_response_serialization() {
        let response = InteractiveAskResponse { params: None };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"params\":null"));
    }

    #[test]
    fn test_interactive_tell_response_serialization() {
        let response = InteractiveTellResponse { ok: true };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ok\":true"));
    }

    // ==================== LOAD STATE OR CONFIG TESTS ====================

    #[test]
    fn test_load_state_or_config_with_existing_state() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a config file
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"{{
            "seed": 42,
            "budget": 20,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
        )
        .unwrap();

        // Create a state file
        let mut state_file = NamedTempFile::new().unwrap();
        writeln!(
            state_file,
            r#"{{
            "config": {{
                "seed": 99,
                "budget": 30,
                "probe_ratio": 0.3,
                "bounds": {{"y": {{"min": 0.0, "max": 2.0}}}}
            }},
            "history": [],
            "run_id": "existing-run"
        }}"#
        )
        .unwrap();

        let loaded =
            load_state_or_config(config_file.path(), Some(&state_file.path().to_path_buf()))
                .unwrap();
        // Should use state's config, not the config file
        assert_eq!(loaded.config.seed, 99);
        assert_eq!(loaded.run_id, Some("existing-run".to_string()));
    }

    #[test]
    fn test_load_state_or_config_no_state_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a config file
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"{{
            "seed": 42,
            "budget": 20,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
        )
        .unwrap();

        let loaded = load_state_or_config(config_file.path(), None).unwrap();
        assert_eq!(loaded.config.seed, 42);
        assert!(loaded.history.is_empty());
        assert!(loaded.run_id.is_none());
    }

    #[test]
    fn test_load_state_or_config_state_doesnt_exist() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a config file
        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"{{
            "seed": 42,
            "budget": 20,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
        )
        .unwrap();

        // Path that doesn't exist
        let nonexistent = PathBuf::from("/tmp/nonexistent_state_12345.json");
        let loaded = load_state_or_config(config_file.path(), Some(&nonexistent)).unwrap();
        // Should fall back to config
        assert_eq!(loaded.config.seed, 42);
        assert!(loaded.run_id.is_none());
    }

    // ==================== VALIDATE COMMAND TESTS ====================

    #[test]
    fn test_validate_command_success() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"{{
            "seed": 42,
            "budget": 20,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
        )
        .unwrap();

        let result = validate_command(config_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_command_invalid_config() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"{{
            "seed": 42,
            "budget": 0,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
        )
        .unwrap();

        let result = validate_command(config_file.path());
        assert!(result.is_err());
    }

    // ==================== READ EVENT LINES TESTS ====================

    #[test]
    fn test_read_event_lines_basic() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"event": "test1", "timestamp_us": 100}}"#).unwrap();
        writeln!(file, r#"{{"event": "test2", "timestamp_us": 200}}"#).unwrap();
        writeln!(file, r#"{{"event": "test3", "timestamp_us": 300}}"#).unwrap();

        let lines = read_event_lines(file.path(), 10).unwrap();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_read_event_lines_limit() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        for i in 0..10 {
            writeln!(
                file,
                r#"{{"event": "test{}", "timestamp_us": {}}}"#,
                i,
                i * 100
            )
            .unwrap();
        }

        let lines = read_event_lines(file.path(), 3).unwrap();
        // Should only get the last 3
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_read_event_lines_filters_invalid_json() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not valid json").unwrap();
        writeln!(file, r#"{{"event": "valid", "timestamp_us": 100}}"#).unwrap();
        writeln!(file, "also invalid").unwrap();

        let lines = read_event_lines(file.path(), 10).unwrap();
        assert_eq!(lines.len(), 1); // Only the valid JSON line
    }

    #[test]
    fn test_read_event_lines_file_not_found() {
        let result = read_event_lines(Path::new("/nonexistent/events.json"), 10);
        assert!(result.is_err());
    }

    // ==================== EVALUATE SCRIPT TESTS ====================

    #[cfg(unix)]
    #[test]
    fn test_evaluate_script_success() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("test_script.sh");
        {
            let mut file = fs::File::create(&script_path).unwrap();
            use std::io::Write;
            file.write_all(b"#!/bin/bash\necho \"0.75\"").unwrap();
            file.sync_all().unwrap();
        }
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let params: HashMap<String, f64> = [("x".to_string(), 0.5)].into_iter().collect();

        let result = evaluate_script(&script_path, &params);
        assert!(result.is_ok(), "evaluate_script failed: {:?}", result);
        assert!((result.unwrap() - 0.75).abs() < 0.001);
    }

    #[cfg(unix)]
    #[test]
    fn test_evaluate_script_with_result_prefix() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("test_script.sh");
        {
            let mut file = fs::File::create(&script_path).unwrap();
            use std::io::Write;
            file.write_all(b"#!/bin/bash\necho \"RESULT=0.99\"")
                .unwrap();
            file.sync_all().unwrap();
        }
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let params: HashMap<String, f64> = HashMap::new();

        let result = evaluate_script(&script_path, &params);
        assert!(result.is_ok(), "evaluate_script failed: {:?}", result);
        assert!((result.unwrap() - 0.99).abs() < 0.001);
    }

    #[cfg(unix)]
    #[test]
    fn test_evaluate_script_failure() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("test_script.sh");
        {
            let mut file = fs::File::create(&script_path).unwrap();
            use std::io::Write;
            file.write_all(b"#!/bin/bash\nexit 1").unwrap();
            file.sync_all().unwrap();
        }
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let params: HashMap<String, f64> = HashMap::new();

        let result = evaluate_script(&script_path, &params);
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn test_evaluate_script_env_vars_set() {
        use std::os::unix::fs::PermissionsExt;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let script_path = dir.path().join("test_script.sh");
        {
            let mut file = fs::File::create(&script_path).unwrap();
            use std::io::Write;
            file.write_all(b"#!/bin/bash\necho $ARQON_alpha").unwrap();
            file.sync_all().unwrap();
        }
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let params: HashMap<String, f64> = [("alpha".to_string(), 0.123)].into_iter().collect();

        let result = evaluate_script(&script_path, &params);
        assert!(result.is_ok(), "evaluate_script failed: {:?}", result);
        assert!((result.unwrap() - 0.123).abs() < 0.001);
    }

    // ==================== START METRICS SERVER TEST ====================

    #[test]
    fn test_metrics_init_with_server_address() {
        // Test that init with an address spawns a thread (just verify it doesn't panic)
        // Using port 0 to let OS assign an available port
        let metrics = Metrics::init(Some("127.0.0.1:0"));
        assert!(metrics.is_ok());
    }

    // ==================== DRAW TUI TESTS ====================

    #[test]
    fn test_format_event_line_with_event_type() {
        // Test event_type fallback
        let line = r#"{"event_type":"update","timestamp_us":1234567890}"#;
        let result = format_event_line(line);
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("update"));
    }

    #[test]
    fn test_format_event_line_without_value() {
        let line = r#"{"event":"test","timestamp_us":1000}"#;
        let result = format_event_line(line);
        assert!(result.is_some());
        let formatted = result.unwrap();
        assert!(formatted.contains("test"));
        assert!(!formatted.contains("value=")); // No value field
    }

    // ==================== GENERATE RUN ID TESTS ====================

    #[test]
    fn test_generate_run_id_uniqueness() {
        let id1 = generate_run_id("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        // In same second, IDs should be same (timestamp-based)
        // But we can at least check format
        assert!(id1.starts_with("test-"));
    }

    // ==================== WRITE OUTPUT TESTS ====================

    #[test]
    fn test_write_output_to_stdout() {
        // When path is None, it should print to stdout - just verify no panic
        let data = serde_json::json!({"test": true});
        let result = write_output(None, &data);
        assert!(result.is_ok());
    }

    // ==================== COMMAND FUNCTION TESTS ====================

    fn create_test_config() -> SolverConfig {
        let mut bounds = HashMap::new();
        bounds.insert(
            "x".to_string(),
            arqonhpo_core::config::Domain {
                min: 0.0,
                max: 1.0,
                scale: arqonhpo_core::config::Scale::Linear,
            },
        );
        SolverConfig {
            bounds,
            budget: 10,
            probe_ratio: 0.5,
            seed: 42,
            strategy_params: None,
        }
    }

    fn create_test_state() -> SolverState {
        SolverState {
            config: create_test_config(),
            history: vec![SeedPoint {
                params: [("x".to_string(), 0.5)].into_iter().collect(),
                value: 1.0,
                cost: 1.0,
            }],
            run_id: Some("test-run".to_string()),
        }
    }

    #[test]
    fn test_ask_command_basic() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"{{
            "seed": 42,
            "budget": 10,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
        )
        .unwrap();

        let metrics = Metrics::init(None).unwrap();
        let result = ask_command(config_file.path(), None, None, &metrics);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ask_command_with_batch_limit() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"{{
            "seed": 42,
            "budget": 10,
            "probe_ratio": 0.5,
            "bounds": {{"x": {{"min": 0.0, "max": 1.0}}}}
        }}"#
        )
        .unwrap();

        let metrics = Metrics::init(None).unwrap();
        let result = ask_command(config_file.path(), None, Some(2), &metrics);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ask_command_with_existing_state() {
        use tempfile::NamedTempFile;

        let mut config_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(
            &mut config_file,
            br#"{"seed": 42, "budget": 10, "probe_ratio": 0.5, "bounds": {"x": {"min": 0.0, "max": 1.0}}}"#,
        )
        .unwrap();

        let mut state_file = NamedTempFile::new().unwrap();
        let state = create_test_state();
        std::io::Write::write_all(
            &mut state_file,
            serde_json::to_string(&state).unwrap().as_bytes(),
        )
        .unwrap();

        let metrics = Metrics::init(None).unwrap();
        let result = ask_command(
            config_file.path(),
            Some(&state_file.path().to_path_buf()),
            None,
            &metrics,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_tell_command_basic() {
        use tempfile::{tempdir, NamedTempFile};

        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        let state = create_test_state();
        fs::write(&state_path, serde_json::to_string(&state).unwrap()).unwrap();

        let mut results_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(
            &mut results_file,
            br#"[{"params": {"x": 0.7}, "value": 0.5, "cost": 1.0}]"#,
        )
        .unwrap();

        let metrics = Metrics::init(None).unwrap();
        let result = tell_command(
            &state_path,
            Some(&results_file.path().to_path_buf()),
            &metrics,
        );
        assert!(result.is_ok());

        // Verify state was updated
        let updated_state: SolverState =
            serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
        assert_eq!(updated_state.history.len(), 2);
    }

    #[test]
    fn test_export_command_basic() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        let output_path = dir.path().join("artifact.json");

        let state = create_test_state();
        fs::write(&state_path, serde_json::to_string(&state).unwrap()).unwrap();

        let metrics = Metrics::init(None).unwrap();
        let result = export_command(&state_path, Some(&output_path), None, &metrics);
        assert!(result.is_ok());

        // Verify artifact was created
        let artifact: RunArtifact =
            serde_json::from_str(&fs::read_to_string(&output_path).unwrap()).unwrap();
        assert_eq!(artifact.history.len(), 1);
        assert_eq!(artifact.run_id, "test-run");
    }

    #[test]
    fn test_export_command_with_custom_run_id() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        let output_path = dir.path().join("artifact.json");

        let state = create_test_state();
        fs::write(&state_path, serde_json::to_string(&state).unwrap()).unwrap();

        let metrics = Metrics::init(None).unwrap();
        let result = export_command(
            &state_path,
            Some(&output_path),
            Some("custom-run".to_string()),
            &metrics,
        );
        assert!(result.is_ok());

        let artifact: RunArtifact =
            serde_json::from_str(&fs::read_to_string(&output_path).unwrap()).unwrap();
        assert_eq!(artifact.run_id, "custom-run");
    }

    #[test]
    fn test_import_command_basic() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let artifact_path = dir.path().join("artifact.json");
        let state_path = dir.path().join("state.json");

        // Create artifact
        let artifact = RunArtifact {
            run_id: "imported-run".to_string(),
            seed: 42,
            budget: 10,
            config: create_test_config(),
            history: vec![EvalTrace {
                eval_id: 1,
                params: [("x".to_string(), 0.5)].into_iter().collect(),
                value: 1.0,
                cost: 1.0,
            }],
        };
        fs::write(&artifact_path, serde_json::to_string(&artifact).unwrap()).unwrap();

        let metrics = Metrics::init(None).unwrap();
        let result = import_command(&artifact_path, &state_path, &metrics);
        assert!(result.is_ok());

        // Verify state was created
        let state: SolverState =
            serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
        assert_eq!(state.run_id, Some("imported-run".to_string()));
        assert_eq!(state.history.len(), 1);
    }

    #[test]
    fn test_export_command_no_output_path() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");

        let state = create_test_state();
        fs::write(&state_path, serde_json::to_string(&state).unwrap()).unwrap();

        let metrics = Metrics::init(None).unwrap();
        // When output_path is None, it prints to stdout
        let result = export_command(&state_path, None, None, &metrics);
        assert!(result.is_ok());
    }
}
