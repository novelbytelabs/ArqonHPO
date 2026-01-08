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

const DASHBOARD_HTML: &str = include_str!("../assets/dashboard.html");
const DASHBOARD_CSS: &str = include_str!("../assets/dashboard.css");
const DASHBOARD_JS: &str = include_str!("../assets/dashboard.js");

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
    println!("Dashboard running at http://{addr}");

    for mut request in server.incoming_requests() {
        let url: &str = request.url();
        let (path, query) = split_query(url);
        let response = match (request.method().as_str(), path) {
            ("GET", "/") => plain_response(DASHBOARD_HTML, "text/html"),
            ("GET", "/assets/dashboard.css") => plain_response(DASHBOARD_CSS, "text/css"),
            ("GET", "/assets/dashboard.js") => plain_response(DASHBOARD_JS, "text/javascript"),
            ("GET", "/api/state") => json_response(load_state_json(state_path, metrics)),
            ("GET", "/api/summary") => json_response(load_summary_json(state_path)),
            ("GET", "/api/events") => {
                let params = parse_query(query);
                json_response(load_events_json(events_path, &params))
            }
            ("GET", "/api/actions") => {
                let params = parse_query(query);
                json_response(load_actions_json(actions_path, &params))
            }
            ("POST", "/api/actions") => json_response(store_action(&mut request, actions_path)),
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

fn plain_response(body: &str, content_type: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(body.as_bytes().to_vec())
        .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type).unwrap())
}

fn json_response(result: Result<serde_json::Value>) -> Response<std::io::Cursor<Vec<u8>>> {
    match result {
        Ok(value) => {
            let data = serde_json::to_vec(&value).unwrap_or_default();
            Response::from_data(data).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], "application/json").unwrap(),
            )
        }
        Err(err) => Response::from_string(err.to_string()).with_status_code(500),
    }
}

fn load_state_json(state_path: &Path, metrics: &Metrics) -> Result<serde_json::Value> {
    let state = load_state(state_path)?;
    metrics.set_history_len(state.history.len());
    Ok(serde_json::to_value(state).into_diagnostic()?)
}

fn load_summary_json(state_path: &Path) -> Result<serde_json::Value> {
    let state = load_state(state_path)?;
    let best = state
        .history
        .iter()
        .map(|entry| entry.value)
        .min_by(|left, right| left.partial_cmp(right).unwrap());
    let latest = state.history.last().map(|entry| entry.value);
    let summary = serde_json::json!({
        "run_id": state.run_id,
        "budget": state.config.budget,
        "history_len": state.history.len(),
        "best": best,
        "latest": latest,
    });
    Ok(summary)
}

fn load_events_json(
    events_path: Option<&PathBuf>,
    params: &HashMap<String, String>,
) -> Result<serde_json::Value> {
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(100);
    let filter = params.get("event").map(String::as_str);
    let search = params.get("q").map(String::as_str);
    let events = if let Some(path) = events_path {
        read_event_values(path, filter, search, limit)?
    } else {
        Vec::new()
    };
    Ok(serde_json::json!({ "events": events }))
}

fn load_actions_json(
    actions_path: Option<&PathBuf>,
    params: &HashMap<String, String>,
) -> Result<serde_json::Value> {
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50);
    let actions = if let Some(path) = actions_path {
        read_jsonl_values(path, limit)?
    } else {
        Vec::new()
    };
    Ok(serde_json::json!({ "actions": actions }))
}

fn store_action(
    request: &mut tiny_http::Request,
    actions_path: Option<&PathBuf>,
) -> Result<serde_json::Value> {
    let Some(path) = actions_path else {
        return Err(miette::miette!("Actions path not configured"));
    };
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .into_diagnostic()?;
    let mut value: serde_json::Value = serde_json::from_str(&body)
        .into_diagnostic()
        .with_context(|| "Invalid JSON body")?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;
    if let serde_json::Value::Object(ref mut map) = value {
        map.entry("timestamp_us".to_string())
            .or_insert(serde_json::Value::Number(timestamp.into()));
    }
    let line = serde_json::to_string(&value).into_diagnostic()?;
    append_line(path, &line)?;
    Ok(serde_json::json!({ "ok": true }))
}

fn append_line(path: &Path, line: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to open {}", path.display()))?;
    writeln!(file, "{}", line).into_diagnostic()?;
    Ok(())
}

fn read_event_values(
    path: &Path,
    filter: Option<&str>,
    search: Option<&str>,
    limit: usize,
) -> Result<Vec<serde_json::Value>> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read events file {}", path.display()))?;
    let mut values = Vec::new();
    for line in contents.lines().rev() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(search) = search {
            if !line.contains(search) {
                continue;
            }
        }
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if let Some(filter) = filter {
            let event_type = value
                .get("event")
                .or_else(|| value.get("event_type"))
                .and_then(|field| field.as_str())
                .unwrap_or("");
            if event_type != filter {
                continue;
            }
        }
        values.push(value);
        if values.len() >= limit {
            break;
        }
    }
    values.reverse();
    Ok(values)
}

fn read_jsonl_values(path: &Path, limit: usize) -> Result<Vec<serde_json::Value>> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read actions file {}", path.display()))?;
    let mut values = Vec::new();
    for line in contents.lines().rev() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        values.push(value);
        if values.len() >= limit {
            break;
        }
    }
    values.reverse();
    Ok(values)
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
