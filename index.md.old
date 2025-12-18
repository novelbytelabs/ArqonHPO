# ArqonHPO

**Adaptive Hyperparameter Optimization for Simulations- **Smart**: The **PCR (Probe-Classify-Refine)** engine automatically switches between Nelder-Mead (for smooth physics) and TPE (for noisy ML) based on landscape analysis.
- **Probe-Gated**: Uses prime-index sampling to "scan" the terrain before committing to a strategy.
- **Zero-Config**: No need to choose a sampler. Just define bounds and budget.

## Features

- 🚀 **300x Faster**: Run 30,000 trials in the time Python solvers run 100.
- 🦀 **Rust Core**: Zero-overhead, deterministic execution.
- 🎯 **PCR Auto-Pilot**: ResidualDecayClassifier detects landscape structure → picks Nelder-Mead (structured) or TPE (chaotic).
- 🐍 **Python Ready**: `pip install arqonhpo`.
- 🔁 **Reproducible**: Seed-controlled, artifact-auditable runs.
- 📐 **Scott's Rule TPE**: Adaptive kernel bandwidth for optimal density estimation.

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
