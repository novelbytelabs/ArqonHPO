# ArqonHPO

**Adaptive Hyperparameter Optimization for Simulations and ML.**

ArqonHPO automatically selects the best optimization strategy based on your objective function's landscape:

- **Smooth, Expensive Simulations?** → Nelder-Mead (fewest evaluations)
- **Noisy, Cheap ML Models?** → TPE (handles variance)

## Features

- 🚀 **300x Faster**: Run 30,000 trials in the time Python solvers run 100.
- 🦀 **Rust Core**: Zero-overhead, deterministic execution.
- 🎯 **Auto-Pilot**: Automatically picks Nelder-Mead or TPE based on your problem.
- 🐍 **Python Ready**: `pip install arqonhpo`.
- 🔁 **Reproducible**: Seed-controlled, artifact-auditable runs.

## ⚡ Multi-Agent Ready
Perfect for **MAS** and **Actor Models** (Rust, Elixir, Go).
*   **Zero Latency**: 2.9ms overhead means you can optimize *inside* your message loop.
*   **No GIL**: Rust core won't block your async Python agents.

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
