# ArqonHPO

**Adaptive Hyperparameter Optimization for Simulations and ML.**

ArqonHPO automatically selects the best optimization strategy based on your objective function's landscape:

- **Smooth, Expensive Simulations?** → Nelder-Mead (fewest evaluations)
- **Noisy, Cheap ML Models?** → TPE (handles variance)

## Features

- 🚀 **Blazing Fast**: **300x faster** than Python-based alternatives (see [Benchmarks](benchmarks.md)).
- 🦀 **Rust Core**: High-performance, deterministic engine.
- 🐍 **Python Bindings**: Simple `pip install arqonhpo`.
- 🎯 **Automatic Mode Selection**: Probe, Classify, Refine.
- 🔁 **Reproducible**: Seed-controlled, artifact-auditable runs.

## Installation

```bash
pip install arqonhpo
```

## Quick Example

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,
    "budget": 100,
    "bounds": {
        "learning_rate": {"min": 1e-5, "max": 1e-1, "scale": "Log"},
        "batch_size": {"min": 16, "max": 256}
    }
}

solver = ArqonSolver(json.dumps(config))

while True:
    batch = solver.ask()
    if batch is None:
        break
    
    results = []
    for params in batch:
        loss = my_objective_function(params)
        results.append({"params": params, "value": loss, "cost": 1.0})
    
    solver.tell(json.dumps(results))
```

[Get Started →](quickstart.md){ .md-button .md-button--primary }
