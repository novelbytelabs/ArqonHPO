# ArqonHPO

**HPO for the AI Era.**

"Optuna optimizes your models. ArqonHPO optimizes your infrastructure."

ArqonHPO is the first hyperparameter optimizer designed for **Machine-Speed** decision making. While traditional tools are built for human timescales (monitoring a training run), ArqonHPO is built for microseconds.

- **Don't put a 300ms brain in a 1ms robot**: If your control loop runs in 10ms, you can't use a Python optimizer that takes 300ms to think. ArqonHPO thinks in **1.7ms**.
- **Systems & Infrastructure**: Tune database knobs, compiler flags, and connection pools in real-time.
- **PCR Auto-Pilot**: Automatically switches between Nelder-Mead (smooth physics) and TPE (noisy ML).

> Modern HPO/NAS tools assume human-time feedback loops: run an experiment, wait, analyze, repeat. But production AI systems live in non-stationary environments—traffic shifts, drift happens, hardware throttles, and constraints change. This work shows that if the optimization loop is fast enough to run at microsecond latency, tuning becomes a control primitive: safe, bounded, auditable adaptation that runs continuously in the runtime, while offline discovery expands the approved search space.

## Features

- 🚀 **300x Faster**: Run 30,000 trials in the time Python solvers run 100.
- 🦀 **Rust Core**: Zero-overhead, deterministic execution.
- 🎯 **PCR Auto-Pilot**: ResidualDecayClassifier detects landscape structure → picks Nelder-Mead (structured) or TPE (chaotic).
- 🐍 **Python Ready**: `pip install arqonhpo`.
- 🔁 **Reproducible**: Seed-controlled, artifact-auditable runs.
- 📐 **Scott's Rule TPE**: Adaptive kernel bandwidth for optimal density estimation.


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

![Rastrigin Performance](reports/phase8/rastrigin_torus__cheap__best_vs_time.png)

> **"Speed is Quality"** - By running 100x more trials in the same time window, ArqonHPO brute-forces complex landscapes that smarter-but-slower algorithms miss.

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
