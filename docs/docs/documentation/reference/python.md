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
