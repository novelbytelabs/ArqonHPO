# Benchmark Report

> **Status:** Preliminary
> **Date:** 2024-12-14

## Executive Summary

ArqonHPO demonstrates a **300x throughput advantage** over Optuna/Ray Tune for cheap functions (<1ms) and maintains **sub-microsecond decision latency** (p99 < 1µs) for control loop integration.

## Methodology

Tests were run on an AWS c6i.4xlarge (Intel Ice Lake) and an Apple M3 Max.

### Protocols

1.  **Latency**: Time to `ask()` + `tell()` in a tight loop.
2.  **Throughput**: Trials per second on a no-op objective.
3.  **Convergence**: Area Under Curve (AUC) on standard HPOBench functions (Rosenbrock, Rastrigin).

## Results

### 1. Decision Latency

| Solver                | p50 (µs) | p99 (µs) | Max (µs) |
| :-------------------- | :------- | :------- | :------- |
| **ArqonHPO (Tier 1)** | **0.12** | **0.85** | **1.2**  |
| Optuna (TPE)          | 2,500    | 15,000   | 45,000   |
| Ray Tune (TPE)        | 15,000   | 120,000  | >200,000 |

### 2. Throughput (Trials/Sec)

![Throughput Plot](throughput_comparison.png)

- **ArqonHPO**: 125,000 trials/sec
- **Optuna**: 450 trials/sec

[Download Raw Data (CSV)](./benchmark_data.csv)
