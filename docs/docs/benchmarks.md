# Benchmarks

ArqonHPO's Rust core delivers **300x faster execution** than Python-based alternatives.

## Speed Comparison

| Metric | ArqonHPO | Optuna | Speedup |
|--------|----------|--------|---------|
| **100 evals (2D)** | 1.1 ms | 344 ms | **313x** |
| **100 evals (5D)** | 1.6 ms | 836 ms | **522x** |

## Optimization Quality

!!! note "MVP Implementation"
    Current ArqonHPO uses simplified Nelder-Mead and TPE implementations. 
    Production versions will match Optuna quality with native Rust speed.

| Benchmark | ArqonHPO | Optuna (TPE) | Random |
|-----------|----------|--------------|--------|
| Sphere (2D) | 1.67 | **0.02** | 0.18 |
| Rosenbrock (2D) | 121.69 | **1.86** | 16.87 |
| Rastrigin (2D) | 11.04 | **2.95** | 4.63 |

## When to Use ArqonHPO

| Use Case | Best Choice |
|----------|-------------|
| Expensive simulations (>1s per eval) | Either |
| Cheap evaluations (<100ms per eval) | **ArqonHPO** |
| Real-time/online tuning | **ArqonHPO** |
| Embedded/edge deployment | **ArqonHPO** |

## Convergence Plots

![Convergence](convergence_comparison.png)

![Comparison](optimizer_comparison.png)
