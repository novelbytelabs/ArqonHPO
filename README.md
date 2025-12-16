# ArqonHPO

[![CI](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml/badge.svg)](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-mkdocs-blue)](https://novelbytelabs.github.io/ArqonHPO/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**HPO for the AI Era.**
"Optuna optimizes your models. ArqonHPO optimizes your infrastructure."

ArqonHPO is the first hyperparameter optimizer designed for **Machine-Speed** decision making.
- **Don't put a 300ms brain in a 1ms robot**: ArqonHPO thinks in **1.7ms**.
- **Real-Time Control**: Tune standard library parameters, connection pools, and MAS agents on the fly.
- **Auto-Pilot**: Automatically selects Nelder-Mead (smooth) or TPE (noisy) based on landscape.

> Modern HPO/NAS tools assume human-time feedback loops: run an experiment, wait, analyze, repeat. But production AI systems live in non-stationary environments—traffic shifts, drift happens, hardware throttles, and constraints change. This work shows that if the optimization loop is fast enough to run at microsecond latency, tuning becomes a control primitive: safe, bounded, auditable adaptation that runs continuously in the runtime, while offline discovery expands the approved search space.

## Features

- 🚀 **300x Faster** - Run 30,000 trials in the time Python solvers run 100.
- 🦀 **Rust Core** - Zero-overhead, deterministic execution.
- 🎯 **Auto-Pilot** - Automatically selects Nelder-Mead (smooth) or TPE (noisy).
- 🐍 **Python Ready** - Simple `pip install arqonhpo`
- 🔁 **Reproducible** - Seed-controlled, artifact-auditable runs

## 🚀 Performance: The 1ms Barrier

**ArqonHPO is built for one thing: Speed.**

In high-throughput optimization—like real-time control, high-frequency trading, or systems tuning—time is your most precious resource. Traditional Python-based optimizers (like Optuna) block your event loop for 300ms+ just to decide the next parameter. ArqonHPO decides in **1.7ms**.

### Tuning Faster Than The Request

Because ArqonHPO's overhead is negligible (~40µs), you can embed optimization directly into **live traffic** logic. Tune your DB connection pool *during* the request handling.

| Metric | ArqonHPO | Optuna (TPE) | Advantage |
|--------|----------|--------------|-----------|
| **Latency (cheap)** | **1.77 ms** | 330.5 ms | **180x speedup** |
| **Rugged Hit Rate** | **93%** | 70% | **Robust Geometry** |
| **Worker Collisions** | **0** | N/A (Requires DB) | **Stateless Sharding** |

![Rastrigin Performance](docs/docs/reports/phase8/rastrigin_torus__cheap__best_vs_time.png)

> **"Speed is Quality"** - By running 100x more trials in the same time window, ArqonHPO brute-forces complex landscapes that smarter-but-slower algorithms miss.

### ⚡ Ideal for Multi-Agent Systems
If you are building a **MAS** with <1ms deadlines, ArqonHPO is the *only* viable choice.
- **Optuna (330ms+)**: Blocks your event loop, causing massive lag.
- **ArqonHPO (1.7ms)**: Fits comfortably inside a single message handler.

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
