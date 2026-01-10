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

#### `ask_one() -> dict | None`

Returns a **single** candidate for online/real-time optimization.

Unlike `ask()` which returns a batch for the PCR workflow, `ask_one()`:
1. Skips Probe/Classify phases
2. Uses TPE strategy directly
3. Returns exactly 1 candidate per call

**Use Case:** Real-time control loops, streaming optimization.

**Example:**

```python
from arqonhpo import ArqonSolver
import json

config = {
    "seed": 42,
    "budget": 100,
    "bounds": {"kp": {"min": 0.1, "max": 10.0}}
}
solver = ArqonSolver(json.dumps(config))

while True:
    candidate = solver.ask_one()
    if candidate is None:
        break
    
    # Evaluate single candidate
    value = evaluate(candidate)
    
    # Immediately feed back
    solver.seed(json.dumps([{
        "params": candidate,
        "value": value,
        "cost": 1.0
    }]))
```

See [Batch vs Online Mode](../concepts/batch_vs_online.md) for details.

---

## ArqonProbe

Stateless Low-Discrepancy Sequence sampler for distributed sampling.

### Constructor

```python
ArqonProbe(config_json: str, seed: int = 42) -> ArqonProbe
```

**Parameters:**

- `config_json`: JSON string with bounds configuration.
- `seed`: RNG seed (default: 42).

### Methods

#### `sample_at(index: int) -> dict`

Generate a single LDS point at the given global index. **Stateless** — the same index always returns the same point.

```python
from arqonhpo import ArqonProbe
import json

config = json.dumps({
    "bounds": {
        "x": {"min": 0, "max": 1},
        "y": {"min": 0, "max": 1}
    }
})

probe = ArqonProbe(config, seed=42)

# Same index → same point
point_0 = probe.sample_at(0)      # {"x": 0.5, "y": 0.5}
point_100 = probe.sample_at(100)  # Different point

# Deterministic
assert probe.sample_at(0) == probe.sample_at(0)  # ✓
```

#### `sample_range(start: int, count: int) -> list[dict]`

Generate a range of LDS points from index `start` to `start + count - 1`.

**Use Case:** Zero-coordination sharding across workers.

```python
# Worker 0: points 0-999
points_w0 = probe.sample_range(0, 1000)

# Worker 1: points 1000-1999
points_w1 = probe.sample_range(1000, 1000)

# No coordination needed — ranges are deterministic and non-overlapping
```

### Sharding Pattern

```python
import json
from arqonhpo import ArqonProbe
from multiprocessing import Pool

config = json.dumps({
    "bounds": {"x": {"min": 0, "max": 1}, "y": {"min": 0, "max": 1}}
})

def worker(worker_id):
    probe = ArqonProbe(config, seed=42)  # Same seed everywhere
    chunk_size = 1000
    start = worker_id * chunk_size
    return probe.sample_range(start, chunk_size)

# All workers produce deterministic, non-overlapping points
with Pool(4) as p:
    all_points = p.map(worker, range(4))
    # 4000 unique, deterministic points
```

See [Determinism](../concepts/determinism.md) for more details.

