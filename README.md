# ArqonHPO

[![CI](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml/badge.svg)](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-mkdocs-blue)](https://novelbytelabs.github.io/ArqonHPO/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Adaptive Hyperparameter Optimization** with automatic strategy selection.

ArqonHPO automatically detects your objective function's landscape and selects the optimal optimization strategy:

- **Smooth, expensive simulations?** → Nelder-Mead (minimizes evaluations)
- **Noisy, cheap ML training?** → TPE (handles variance)

## Features

- 🦀 **Rust Core** - High-performance engine (**300x faster than Python solvers**)
- 🐍 **Python Bindings** - Simple `pip install arqonhpo`
- 🎯 **Auto Strategy Selection** - Probe → Classify → Refine
- 🔁 **Reproducible** - Seed-controlled, artifact-auditable runs
- 📊 **Tracing** - Built-in observability spans

## 🚀 Performance

ArqonHPO is built for high-throughput and low-latency optimization. Benchmarks show a **~300x speedup** compared to standard Python TPE implementations.

| Metric | ArqonHPO | Optuna (TPE) | Speedup |
|--------|----------|--------------|---------|
| **Average Overhead** | 2.9 ms | 846.4 ms | **297x faster** |

> Faster execution means you can run more trials in less time, or deploy optimization in real-time control loops where latency matters.

## Installation

```bash
pip install arqonhpo
```

Or build from source:

```bash
git clone https://github.com/novelbytelabs/ArqonHPO.git
cd ArqonHPO
pip install maturin
maturin develop -m bindings/python/Cargo.toml
```

## Quick Start

```python
import json
from arqonhpo import ArqonSolver

# Define your objective function
def objective(params):
    x, y = params["x"], params["y"]
    return (x - 2)**2 + (y + 1)**2  # Minimum at (2, -1)

# Configure solver
config = {
    "seed": 42,
    "budget": 50,
    "bounds": {
        "x": {"min": -10.0, "max": 10.0},
        "y": {"min": -10.0, "max": 10.0}
    }
}

solver = ArqonSolver(json.dumps(config))

# Optimization loop
best = {"value": float('inf')}
while (batch := solver.ask()) is not None:
    results = []
    for params in batch:
        value = objective(params)
        if value < best["value"]:
            best = {"params": params, "value": value}
        results.append({"params": params, "value": value, "cost": 1.0})
    solver.tell(json.dumps(results))

print(f"Best: {best['params']} -> {best['value']:.4f}")
```

## Documentation

- [**Quickstart**](https://novelbytelabs.github.io/ArqonHPO/quickstart/) - Get running in 5 minutes
- [**Cookbook**](https://novelbytelabs.github.io/ArqonHPO/cookbook/) - Sim tuning & ML tuning recipes
- [**API Reference**](https://novelbytelabs.github.io/ArqonHPO/reference/python/)

## Architecture

```
┌─────────────┐     ┌─────────────────────────────────────┐
│   Python    │────▶│          arqonhpo._internal         │
│   Client    │     │             (PyO3)                  │
└─────────────┘     └────────────────┬────────────────────┘
                                     │
                    ┌────────────────▼────────────────────┐
                    │          arqonhpo-core              │
                    │  ┌─────────────────────────────┐   │
                    │  │    Solver State Machine     │   │
                    │  │  Probe→Classify→Refine      │   │
                    │  └─────────────────────────────┘   │
                    │  ┌─────────────────────────────┐   │
                    │  │       Strategies            │   │
                    │  │  • NelderMead (Structured)  │   │
                    │  │  • TPE (Chaotic)            │   │
                    │  └─────────────────────────────┘   │
                    └─────────────────────────────────────┘
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.
