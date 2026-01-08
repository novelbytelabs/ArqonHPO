# Observability

## Structured Logs

The CLI uses `tracing` and emits structured logs to stderr. Enable JSON output with:

```bash
arqonhpo --log-format json --log-level info <command>
```

Common fields include:

- `command`
- `config`
- `state`
- `artifact`

## Metrics

Enable Prometheus metrics with:

```bash
arqonhpo --metrics-addr 127.0.0.1:9898 <command>
```

Metrics emitted by the CLI:

- `arqonhpo_ask_calls`
- `arqonhpo_tell_calls`
- `arqonhpo_candidates_emitted`
- `arqonhpo_results_ingested`
- `arqonhpo_history_len`
- `arqonhpo_eval_seconds`

## TUI Dashboard

The TUI reads the solver `--state` file and optionally an events JSONL file:

```bash
arqonhpo tui --state state.json
```

Quit with `q` or `Esc`.

## Web Dashboard

Launch the dashboard server:

```bash
arqonhpo dashboard --state state.json --addr 127.0.0.1:3030
```

If the `arqonhpo` command is missing, install the Rust CLI binary:

```bash
cargo install --path crates/cli --bin arqonhpo-cli
```

Optional files:

- `--events audit.jsonl` (audit stream)
- `--actions actions.jsonl` (queues control actions)

### Action Payloads

The dashboard posts JSON to `--actions` for human-in-the-loop control. Example payloads:

```json
{"action":"pause","reason":"Investigating regression"}
```

```json
{"action":"resume","reason":"Safe to continue"}
```

```json
{"action":"rollback","reason":"Rollback to last stable"}
```

Each action is stored as JSONL with an added `timestamp_us` field.

### Security Note

The dashboard server is unauthenticated and intended for localhost use only. Run it behind a secured proxy if you need remote access.
