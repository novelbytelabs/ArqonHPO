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
