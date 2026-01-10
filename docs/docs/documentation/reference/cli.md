# CLI Reference

ArqonHPO provides a command-line interface for batch optimization, interactive ask/tell flows, and observability.

---

## Installation

```bash
# From cargo
cargo install --path crates/cli --bin arqonhpo-cli

# Or use Python wrapper
pip install arqonhpo
python -m arqonhpo --help
```

---

## Commands

### `run` — Batch Optimization

Run a complete optimization loop with an evaluation script.

```bash
arqonhpo run --config config.json --script ./evaluate.sh --state state.json
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--config` | ✓ | — | Path to config JSON |
| `--script` | ✓ | — | Path to evaluation script |
| `--state` | ✗ | — | Path to state file (for resume) |

**Evaluation Script:**

The script receives parameters as environment variables prefixed with `ARQON_`:

```bash
#!/bin/bash
# evaluate.sh
echo "RESULT=$(python my_sim.py --x=$ARQON_x --y=$ARQON_y)"
```

The script must print `RESULT=<float>` to stdout.

---

### `ask` — Request Candidates

Request the next batch of candidate parameters.

```bash
arqonhpo ask --config config.json --state state.json --batch 4
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--config` | ✓ | — | Path to config JSON |
| `--state` | ✗ | — | Path to state file |
| `--batch` | ✗ | 1 | Number of candidates to request |

**Output (stdout):**

```json
[
  {"x": 0.4, "y": -1.2},
  {"x": 0.5, "y": -1.0}
]
```

Returns empty array `[]` if budget exhausted.

---

### `tell` — Report Results

Report evaluation results back to the solver.

```bash
arqonhpo tell --state state.json --results results.json
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--state` | ✓ | — | Path to state file |
| `--results` | ✗ | stdin | Path to results JSON (or read from stdin) |

**Results Schema:**

```json
[
  {
    "params": {"x": 0.4, "y": -1.2},
    "value": 0.12,
    "cost": 1.0
  }
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `params` | object | ✓ | Parameter values |
| `value` | float | ✓ | Objective value (minimize) |
| `cost` | float | ✗ | Evaluation cost (default: 1.0) |

---

### `interactive` — JSONL Mode

Start an interactive JSONL session over stdin/stdout.

```bash
arqonhpo interactive --config config.json --state state.json
```

**Protocol:**

```
→ {"cmd":"ask","batch":2}
← {"params":[{"x":0.4,"y":-1.2},{"x":0.5,"y":-1.0}]}

→ {"cmd":"tell","results":[{"params":{"x":0.4,"y":-1.2},"value":0.12,"cost":1.0}]}
← {"ok":true}
```

**Commands:**

| Command | Fields | Response |
|---------|--------|----------|
| `{"cmd":"ask","batch":N}` | `batch` (optional) | `{"params":[...]}` or `{"done":true}` |
| `{"cmd":"tell","results":[...]}` | `results` (required) | `{"ok":true}` |
| `{"cmd":"status"}` | — | `{"history_len":N,"budget_remaining":M}` |

---

### `validate` — Validate Config

Check a config file for errors without running optimization.

```bash
arqonhpo validate --config config.json
```

**Output:**

```
✓ Config valid: 2 parameters, budget=100
```

Or on error:

```
✗ Error: bounds.x.min must be less than bounds.x.max
```

---

### `export` — Export Artifact

Export solver state as a portable artifact for replay.

```bash
arqonhpo export --state state.json --output artifact.json --run-id my-experiment
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--state` | ✓ | — | Path to state file |
| `--output` | ✗ | stdout | Output path |
| `--run-id` | ✗ | UUID | Run identifier |

**Artifact Schema:**

```json
{
  "run_id": "my-experiment",
  "timestamp": "2026-01-09T12:00:00Z",
  "config": { ... },
  "history": [
    {"eval_id": 0, "params": {...}, "value": 0.5, "cost": 1.0}
  ]
}
```

---

### `import` — Import Artifact

Import a previously exported artifact to resume or replay.

```bash
arqonhpo import --artifact artifact.json --state state.json
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--artifact` | ✓ | — | Path to artifact JSON |
| `--state` | ✓ | — | Output state file path |

---

### `tui` — Terminal Dashboard

Launch the terminal UI for real-time monitoring.

```bash
arqonhpo tui --state state.json --events events.jsonl --refresh-ms 500
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--state` | ✓ | — | Path to state file |
| `--events` | ✗ | — | Path to events log |
| `--refresh-ms` | ✗ | 500 | Refresh interval |

See [TUI Reference](tui.md) for keybindings and interface details.

---

### `dashboard` — Web Dashboard

Launch the web-based monitoring dashboard.

```bash
arqonhpo dashboard --state state.json --addr 127.0.0.1:3030
```

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--state` | ✓ | — | Path to state file |
| `--events` | ✗ | — | Path to events log |
| `--actions` | ✗ | — | Path to actions log |
| `--addr` | ✗ | 127.0.0.1:3030 | Bind address |

See [Dashboard Reference](dashboard.md) for REST API endpoints.

---

## Global Options

| Flag | Values | Default | Description |
|------|--------|---------|-------------|
| `--log-format` | `pretty`, `json` | `pretty` | Log output format |
| `--log-level` | `error`, `warn`, `info`, `debug`, `trace` | `info` | Log verbosity |
| `--metrics-addr` | `HOST:PORT` | — | Prometheus metrics endpoint |

**Example:**

```bash
arqonhpo --log-format json --log-level debug --metrics-addr 127.0.0.1:9898 run ...
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ARQON_LOG_LEVEL` | Override log level |
| `ARQON_LOG_FORMAT` | Override log format |
| `ARQON_<param>` | Parameter value (set during script execution) |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error (invalid config, IO error) |
| `2` | Config validation error |
| `3` | State file error (corrupt, incompatible) |
| `10` | Budget exhausted |
| `20` | Script execution failed |
| `130` | Interrupted (SIGINT) |

---

## Config File Schema

```json
{
  "seed": 42,
  "budget": 100,
  "bounds": {
    "x": {"min": -5, "max": 5, "scale": "Linear"},
    "y": {"min": 0.01, "max": 100, "scale": "Log"}
  },
  "probe_ratio": 0.2,
  "batch_size": 4,
  "strategy": null,
  "strategy_params": {}
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `seed` | int | ✓ | — | RNG seed for reproducibility |
| `budget` | int | ✓ | — | Max evaluations |
| `bounds` | object | ✓ | — | Parameter bounds |
| `bounds.<name>.min` | float | ✓ | — | Minimum value |
| `bounds.<name>.max` | float | ✓ | — | Maximum value |
| `bounds.<name>.scale` | string | ✗ | `Linear` | `Linear`, `Log`, `Periodic` |
| `probe_ratio` | float | ✗ | 0.2 | Fraction for probing |
| `batch_size` | int | ✗ | 4 | Candidates per ask |
| `strategy` | string | ✗ | auto | Force strategy: `nelder_mead`, `multi_start_nm`, `tpe` |
| `strategy_params` | object | ✗ | {} | Strategy-specific params |

---

## State File

The state file persists solver state between commands. It's updated by `ask` and `tell`.

> [!WARNING]
> Do not manually edit state files. Use `import`/`export` for portability.

---

## Next Steps

- [TUI Reference](tui.md) — Terminal monitoring
- [Dashboard Reference](dashboard.md) — Web monitoring
- [Python API](python.md) — Programmatic access
