# Python API Reference

::: arqonhpo

## ArqonSolver

The main entry point for optimization.

### Constructor

```python
ArqonSolver(config_json: str) -> ArqonSolver
```

**Parameters:**

- `config_json`: JSON string with solver configuration.

**Config Schema:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `seed` | int | ✓ | - | RNG seed for reproducibility |
| `budget` | int | ✓ | - | Max number of evaluations |
| `bounds` | dict | ✓ | - | Parameter bounds (see below) |
| `probe_ratio` | float | ✗ | 0.2 | Fraction of budget for probing |
| `strategy_params` | dict | ✗ | null | Strategy-specific config |

**Bounds Format:**

```json
{
  "param_name": {
    "min": 0.0,
    "max": 1.0,
    "scale": "Linear"  // or "Log"
  }
}
```

### Methods

#### `ask() -> list[dict] | None`

Returns the next batch of candidate parameters, or `None` if optimization is complete.

#### `tell(results_json: str) -> None`

Report evaluation results back to the solver.

**Results Schema:**

```json
[
  {
    "eval_id": 0,
    "params": {"x": 1.0, "y": 2.0},
    "value": 0.5,
    "cost": 1.0
  }
]
```

#### `seed(seed_json: str) -> None`

Inject historical evaluations into the solver for warm-starting.
The solver assigns internal `eval_id`s automatically.

**Use Cases:**

- **Warm-starting**: Resume optimization from a previous run's data
- **Streaming/Online optimization**: External systems generate evaluations

**Seed Data Schema:**

```json
[
  {
    "params": {"x": 1.0, "y": 2.0},
    "value": 0.5,
    "cost": 1.0
  }
]
```

Note: Unlike `tell()`, the seed data does **not** require `eval_id` fields.

**Example:**

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,
    "budget": 100,
    "bounds": {"x": {"min": -5.0, "max": 5.0, "scale": "Linear"}},
    "probe_ratio": 0.2
}
solver = ArqonSolver(json.dumps(config))

# Seed with historical data from a previous run
historical = [
    {"params": {"x": 0.5}, "value": 10.0, "cost": 1.0},
    {"params": {"x": 1.5}, "value": 5.0, "cost": 1.0}
]
solver.seed(json.dumps(historical))

# Next ask() will be informed by seeded data
batch = solver.ask()
```

!!! tip "Probe Budget"
    To trigger phase transition to Classify→Refine, seed at least 
    `budget × probe_ratio` points (default: 20% of budget).

#### `get_history_len() -> int`

Returns the current number of evaluations in the solver's history.
Useful for verifying seeding or tracking progress.
