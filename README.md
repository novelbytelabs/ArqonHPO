# ArqonHPO

[![CI](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml/badge.svg)](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-mkdocs-blue)](https://novelbytelabs.github.io/ArqonHPO/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Adaptive Hyperparameter Optimization** with automatic strategy selection and real-time parameter tuning.

ArqonHPO automatically detects your objective function's landscape and selects the optimal optimization strategy:

- **Smooth, expensive simulations?** → Nelder-Mead (minimizes evaluations)
- **Noisy, cheap ML training?** → TPE (handles variance)
- **Live production systems?** → **NEW!** Adaptive Engine (microsecond-latency tuning)

## ✨ What's New in v0.2.0

- 🔥 **Adaptive Engine** - Real-time SPSA optimizer with µs latency
- 🛡️ **Hot-Path Enforcement** - Constitution-mandated safety (no HashMap in critical paths)
- ⚡ **109ns T1 Apply** - Sub-microsecond config updates
- 🔒 **Safety Executor** - Guardrails prevent unbounded changes
- 📊 **Audit Queue** - Lock-free, non-blocking event logging

## Features

- 🚀 **300x Faster** - Run 30,000 trials in the time Python solvers run 100
- 🦀 **Rust Core** - Zero-overhead, deterministic execution
- 🎯 **Auto-Pilot** - Automatically selects Nelder-Mead (smooth) or TPE (noisy)
- 🐍 **Python Ready** - Simple `pip install arqonhpo`
- 🔁 **Reproducible** - Seed-controlled, artifact-auditable runs
- ⚙️ **Adaptive Engine** - Live parameter tuning for production systems

## 🚀 Performance

**ArqonHPO is built for one thing: Speed.**

| Metric | ArqonHPO | Optuna (TPE) | Advantage |
|--------|----------|--------------|-----------|
| **100 Trials (2D)** | 1.1 ms | 344 ms | **313x faster** |
| **Throughput** | ~33,000/sec | ~300/sec | **100x volume** |
| **T1 Apply** | 109 ns | N/A | **Hot path optimized** |
| **T2 Decision** | 200 ns | N/A | **Microsecond latency** |

> **"Speed is King"** - When evaluations are cheap (<10ms), ArqonHPO allows you to brute-force the problem with massive volume, beating smarter but slower algorithms.

### ⚡ Ideal for Multi-Agent Systems
If you are building a **MAS** with <1ms deadlines, ArqonHPO is the *only* viable choice.

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
                                     │
                    ┌────────────────▼────────────────────┐
                    │           hotpath (v0.2.0)          │
                    │  ┌─────────────────────────────┐   │
                    │  │  Tier 2: AdaptiveEngine     │   │
                    │  │  • SPSA Optimizer           │   │
                    │  │  • Telemetry Ingestion      │   │
                    │  └─────────────────────────────┘   │
                    │  ┌─────────────────────────────┐   │
                    │  │  Tier 1: SafetyExecutor     │   │
                    │  │  • Guardrails & Rollback    │   │
                    │  │  • Audit Queue (lock-free)  │   │
                    │  └─────────────────────────────┘   │
                    └─────────────────────────────────────┘
```

## Constitution

ArqonHPO is developed under a **Constitution** - a living document that codifies invariants, contracts, and quality standards. Key principles:

- **Hot-Path Enforcement (VIII.3)**: No `HashMap` in Tier 1/2 code
- **Timing Contracts (VIII.4)**: T1 ≤ 100µs, T2 ≤ 1000µs (p99)
- **Audit Completeness**: No silent drops, lock-free queuing

See [`.specify/memory/constitution.md`](.specify/memory/constitution.md) for the full document.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

