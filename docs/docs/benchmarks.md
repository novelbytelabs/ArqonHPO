# ArqonHPO Benchmarks

Comprehensive comparison against Optuna (TPE) and Random Search across multiple optimization landscapes.

> **Key Finding**: ArqonHPO delivers **~300x faster execution speed** than Python-based alternatives, making it ideal for high-throughput and real-time optimization.

## 🚀 Speed Comparison

| Metric | ArqonHPO | Optuna (TPE) | Advantage |
|--------|----------|--------------|-----------|
| **Average Overhead** | 2.9 ms | 846.4 ms | **297x faster** |

![Speedup Chart](speedup_comparison.png)

## 📊 US1: Smooth Functions
*Targeting expensive simulations (Nelder-Mead use case)*

Benchmarks include Sphere, Rosenbrock, Beale, Booth, and Quadratic functions up to 20 dimensions.

| Benchmark | ArqonHPO | Optuna-TPE | Random | Result |
|-----------|----------|------------|--------|--------|
| Sphere-2D | 1.11 | **0.02** | 0.18 | Optuna leads |
| Beale-2D | 0.76 | **0.02** | 0.30 | Optuna leads |
| Rosenbrock-2D | 12.33 | **2.94** | 8.33 | Optuna leads |

![US1 Comparison](us1_smooth_functions_comparison.png)

## 📉 US2: Noisy/Multimodal Functions
*Targeting ML hyperparameter tuning (TPE use case)*

Benchmarks include Rastrigin, Ackley, Levy, Griewank, and Schwefel functions with multiple local minima.

| Benchmark | ArqonHPO | Optuna-TPE | Random | Result |
|-----------|----------|------------|--------|--------|
| Ackley-2D | 4.36 | **0.47** | 1.70 | Optuna leads |
| Rastrigin-2D | 13.83 | **2.54** | 3.99 | Optuna leads |
| NoisySphere-5D | 18.24 | **0.92** | 7.41 | Optuna leads |

![US2 Comparison](us2_noisy_multimodal_comparison.png)

## 📈 Scaling Analysis

Performance scaling with increasing dimensionality (2D → 5D → 10D → 20D).

![Scaling Analysis](scaling_analysis.png)

## 🎯 Recommendations

| Use Case | Recommended Tool | Why? |
|----------|------------------|------|
| **Real-time / Online Tuning** | **ArqonHPO** | < 3ms latency allows tuning inside control loops. |
| **Cheap Objectives (<100ms)** | **ArqonHPO** | Python overhead dominates with other tools. |
| **Embedded / Edge** | **ArqonHPO** | Native binary, no Python runtime required. |
| **Expensive Simulations (>1s)** | **Optuna** | Higher sample efficiency justifies the overhead. |
| **Maximum Accuracy** | **Optuna** | More mature algorithms (currently). |

## 🏗️ Future Roadmap
ArqonHPO v0.2+ will focus on closing the loop on sample efficiency (accuracy) while maintaining the 300x speed advantage:
1. **Adaptive Nelder-Mead** with restart heuristics.
2. **Full TPE Implementation** with Parzen Estimators matching Optuna's logic.
3. **CMA-ES** for tough non-convex problems.
