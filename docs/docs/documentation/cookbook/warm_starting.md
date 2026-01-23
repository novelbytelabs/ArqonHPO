# Warm-Starting with Seed Data

This guide shows how to use the `seed()` method to warm-start the solver with historical evaluation data.

## Use Cases

- **Resume optimization**: Continue from a previous run's data
- **Transfer learning**: Use data from similar optimization problems
- **Online/streaming optimization**: External systems generate evaluations

## Basic Usage

```python
import json
from arqonhpo import ArqonSolver

# Configure the solver
config = {
    "seed": 42,
    "budget": 100,
    "bounds": {
        "learning_rate": {"min": 1e-5, "max": 1e-1, "scale": "Log"},
        "batch_size": {"min": 16, "max": 256, "scale": "Linear"}
    },
    "probe_ratio": 0.2
}
solver = ArqonSolver(json.dumps(config))

# Historical data from a previous run
historical_data = [
    {"params": {"learning_rate": 0.001, "batch_size": 32}, "value": 0.85, "cost": 1.0},
    {"params": {"learning_rate": 0.01, "batch_size": 64}, "value": 0.78, "cost": 1.0},
    {"params": {"learning_rate": 0.0001, "batch_size": 128}, "value": 0.92, "cost": 1.0},
]

# Seed the solver with historical data
solver.seed(json.dumps(historical_data))

# Verify the data was added
print(f"History length: {solver.get_history_len()}")  # Output: 3

# Continue optimization - solver will use seeded data
batch = solver.ask()
```

## Warm-Starting from a Previous Run

```python
import json
from pathlib import Path
from arqonhpo import ArqonSolver

def load_previous_run(run_path: str) -> list:
    """Load evaluation history from a previous run."""
    with open(run_path) as f:
        artifact = json.load(f)

    # Convert EvalTrace format to SeedPoint format
    return [
        {
            "params": eval["params"],
            "value": eval["value"],
            "cost": eval["cost"]
        }
        for eval in artifact["history"]
    ]

# Resume from previous run
config = {
    "seed": 42,
    "budget": 200,  # Continue with more budget
    "bounds": {
        "x": {"min": -5.0, "max": 5.0, "scale": "Linear"},
        "y": {"min": -5.0, "max": 5.0, "scale": "Linear"}
    },
    "probe_ratio": 0.2
}

solver = ArqonSolver(json.dumps(config))

# Seed with previous run data
previous_data = load_previous_run("run_artifact.json")
solver.seed(json.dumps(previous_data))

# Continue optimization
while True:
    batch = solver.ask()
    if batch is None:
        break

    # Evaluate and tell...
```

## Streaming Optimization

For online systems where evaluations arrive asynchronously:

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,
    "budget": 1000,
    "bounds": {"x": {"min": 0.0, "max": 1.0, "scale": "Linear"}},
    "probe_ratio": 0.2
}
solver = ArqonSolver(json.dumps(config))

def on_external_evaluation(params: dict, value: float, cost: float):
    """Called when an external system completes an evaluation."""
    seed_point = [{"params": params, "value": value, "cost": cost}]
    solver.seed(json.dumps(seed_point))
    print(f"Seeded evaluation: {params} -> {value}")

# External evaluations can arrive any time
on_external_evaluation({"x": 0.5}, 10.0, 1.0)
on_external_evaluation({"x": 0.25}, 8.5, 1.0)

# Solver history now includes external data
print(f"Total evaluations: {solver.get_history_len()}")
```

## Key Points

!!! tip "Probe Budget"
The solver needs `budget × probe_ratio` evaluations (default: 20%)
to transition from Probe to Classify phase. Seed enough data to
meet this threshold if you want to skip probing.

!!! note "eval_id Not Required"
Unlike `tell()`, the `seed()` method does not require `eval_id`
in the input data. The solver assigns IDs automatically.

!!! warning "Bounds Consistency"
Ensure seeded parameter values are within the configured bounds.
The solver does not validate seeded data against bounds.
