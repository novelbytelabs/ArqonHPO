# CLI Reference

ArqonHPO provides a command-line interface for batch optimization, interactive ask/tell flows, and observability.

!!! note "Phase 2 MVP Scope"
    The CLI described here tracks Phase 2 deliverables and may evolve as implementation lands.

## Commands

### Batch runs

```bash
arqonhpo run --config config.json --script ./evaluate.sh --state state.json
```

### Ask/Tell

```bash
arqonhpo ask --config config.json --state state.json --batch 4
arqonhpo tell --state state.json --results results.json
```

### Interactive

```bash
arqonhpo interactive --config config.json --state state.json
```

### Validate

```bash
arqonhpo validate --config config.json
```

### Export/Import

```bash
arqonhpo export --state state.json --output artifact.json
arqonhpo import --artifact artifact.json --state state.json
```

### TUI

```bash
arqonhpo tui --state state.json
```

### Dashboard

```bash
arqonhpo dashboard --state state.json --addr 127.0.0.1:3030
```

## Global Options

- `--log-format` (`pretty` or `json`)
- `--log-level` (e.g. `info`, `debug`)
- `--metrics-addr` (Prometheus endpoint, e.g. `127.0.0.1:9898`)

## Config File

`config.json`:

```json
{
  "seed": 42,
  "budget": 100,
  "bounds": {
    "x": {"min": -5, "max": 5, "scale": "linear"},
    "y": {"min": -5, "max": 5, "scale": "linear"}
  },
  "probe_ratio": 0.2,
  "batch_size": 4,
  "strategy_params": {
    "alpha": 0.1
  }
}
```

### Config schema (MVP)

- `seed` (int, required)
- `budget` (int > 0, required)
- `bounds` (object, required)
- `bounds.<name>.min` / `bounds.<name>.max` (numbers, required)
- `bounds.<name>.scale` (`linear` | `log` | `periodic`, optional)
- `probe_ratio` (0–1, optional)
- `batch_size` (int, optional)
- `strategy_params` (object, optional)

## Batch Evaluation Script

The CLI will call your script with parameters as environment variables:

```bash
#!/bin/bash
# evaluate.sh
echo "RESULT=$(python my_simulation.py --x=$ARQON_x --y=$ARQON_y)"
```

## Ask Output

`arqonhpo ask` writes a JSON array of candidates to stdout:

```json
[
  {"x": 0.4, "y": -1.2},
  {"x": 0.5, "y": -1.0}
]
```

## Tell Input

`results.json`:

```json
[
  {"params": {"x": 0.4, "y": -1.2}, "value": 0.12, "cost": 1.0},
  {"params": {"x": 0.5, "y": -1.0}, "value": 0.10, "cost": 1.0}
]
```

## Interactive Mode

The interactive mode is JSONL over stdin/stdout:

```json
{"cmd":"ask","batch":2}
```

```json
{"params":[{"x":0.4,"y":-1.2},{"x":0.5,"y":-1.0}]}
```

```json
{"cmd":"tell","results":[{"params":{"x":0.4,"y":-1.2},"value":0.12,"cost":1.0}]}
```

```json
{"ok":true}
```

## State File

Use `--state state.json` to persist solver state between `ask` and `tell` calls.

## Exported Artifact

`export` writes a `RunArtifact` JSON file containing `config`, `history`, and identifiers for replay.
