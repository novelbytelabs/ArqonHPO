#!/usr/bin/env python3
"""
Comprehensive Probe Benchmark: A/B Testing Legacy vs New Probe

Compares:
1. ArqonHPO with NEW PrimeSqrtSlopesRotProbe
2. ArqonHPO with LEGACY PrimeIndexProbe (for A/B)
3. Optuna (TPE)
4. Random Search

Uses torus-shift protocol to prevent lucky alignment.
"""

import json
import time
import random
import statistics
from dataclasses import dataclass
from typing import Callable, Dict, List, Tuple, Optional
import numpy as np

# Try to import ArqonHPO
try:
    from arqonhpo import ArqonSolver
    HAS_ARQON = True
except ImportError:
    HAS_ARQON = False
    print("Warning: ArqonHPO not installed.")

# Try to import Optuna
try:
    import optuna
    optuna.logging.set_verbosity(optuna.logging.WARNING)
    HAS_OPTUNA = True
except ImportError:
    HAS_OPTUNA = False
    print("Warning: Optuna not installed.")


# =============================================================================
# Benchmark Functions (on unit cube, shifted)
# =============================================================================

def hartmann6(x: np.ndarray) -> float:
    """Hartmann 6D function. x in [0,1]^6. Global min ~-3.32237."""
    alpha = np.array([1.0, 1.2, 3.0, 3.2])
    A = np.array([
        [10, 3, 17, 3.5, 1.7, 8],
        [0.05, 10, 17, 0.1, 8, 14],
        [3, 3.5, 1.7, 10, 17, 8],
        [17, 8, 0.05, 10, 0.1, 14],
    ], dtype=float)
    P = 1e-4 * np.array([
        [1312, 1696, 5569, 124, 8283, 5886],
        [2329, 4135, 8307, 3736, 1004, 9991],
        [2348, 1451, 3522, 2883, 3047, 6650],
        [4047, 8828, 8732, 5743, 1091, 381],
    ], dtype=float)
    inner = np.sum(A * (x[None, :] - P) ** 2, axis=-1)
    return -np.sum(alpha * np.exp(-inner))


def rosenbrock(x: np.ndarray) -> float:
    """Rosenbrock. x in [0,1]^d mapped to [-2.048, 2.048]^d. Global min at (1,...,1)."""
    x = (x * 2 - 1) * 2.048
    return np.sum(100 * (x[1:] - x[:-1]**2)**2 + (1 - x[:-1])**2)


def rastrigin(x: np.ndarray) -> float:
    """Rastrigin. x in [0,1]^d mapped to [-5.12, 5.12]^d. Global min at origin."""
    x = (x * 2 - 1) * 5.12
    d = len(x)
    return 10 * d + np.sum(x**2 - 10 * np.cos(2 * np.pi * x))


def sphere(x: np.ndarray) -> float:
    """Sphere. x in [0,1]^d mapped to [-5, 5]^d. Global min at origin."""
    x = (x * 2 - 1) * 5.0
    return np.sum(x**2)


BENCHMARKS = {
    "Sphere (5D)": {"fn": sphere, "dims": 5},
    "Rosenbrock (5D)": {"fn": rosenbrock, "dims": 5},
    "Rastrigin (5D)": {"fn": rastrigin, "dims": 5},
    "Hartmann (6D)": {"fn": hartmann6, "dims": 6},
}


# =============================================================================
# Torus-shifted objective wrapper
# =============================================================================

def make_shifted_objective(fn: Callable, shift: np.ndarray) -> Callable:
    """Wrap objective with torus shift to prevent lucky alignment."""
    def shifted(params: Dict[str, float]) -> float:
        # Extract values in sorted key order
        keys = sorted(params.keys())
        x = np.array([params[k] for k in keys])
        # Apply torus shift
        x_shifted = (x + shift) % 1.0
        return fn(x_shifted)
    return shifted


# =============================================================================
# Optimizers
# =============================================================================

@dataclass
class BenchmarkResult:
    optimizer: str
    benchmark: str
    best_value: float
    evaluations: int
    wall_time_ms: float
    shift_id: int
    seed: int


def run_arqonhpo(fn: Callable, dims: int, budget: int, seed: int) -> BenchmarkResult:
    """Run ArqonHPO (uses new probe by default)."""
    config = {
        "seed": seed,
        "budget": budget,
        "bounds": {f"x{i}": {"min": 0.0, "max": 1.0} for i in range(dims)},
        "probe_ratio": 0.2
    }
    
    solver = ArqonSolver(json.dumps(config))
    best = float('inf')
    evals = 0
    
    start = time.perf_counter()
    
    while True:
        candidates = solver.ask()
        if candidates is None:
            break
        
        results = []
        for params in candidates:
            value = fn(params)
            if value < best:
                best = value
            evals += 1
            results.append({"eval_id": evals, "params": params, "value": value, "cost": 1.0})
        
        solver.tell(json.dumps(results))
        
        if evals >= budget:
            break
    
    elapsed_ms = (time.perf_counter() - start) * 1000
    
    return BenchmarkResult(
        optimizer="ArqonHPO (New Probe)",
        benchmark="",
        best_value=best,
        evaluations=evals,
        wall_time_ms=elapsed_ms,
        shift_id=0,
        seed=seed
    )


def run_optuna(fn: Callable, dims: int, budget: int, seed: int) -> BenchmarkResult:
    """Run Optuna TPE."""
    best = float('inf')
    
    def objective(trial):
        nonlocal best
        params = {f"x{i}": trial.suggest_float(f"x{i}", 0.0, 1.0) for i in range(dims)}
        value = fn(params)
        if value < best:
            best = value
        return value
    
    sampler = optuna.samplers.TPESampler(seed=seed)
    study = optuna.create_study(direction="minimize", sampler=sampler)
    
    start = time.perf_counter()
    study.optimize(objective, n_trials=budget, show_progress_bar=False)
    elapsed_ms = (time.perf_counter() - start) * 1000
    
    return BenchmarkResult(
        optimizer="Optuna (TPE)",
        benchmark="",
        best_value=study.best_value,
        evaluations=budget,
        wall_time_ms=elapsed_ms,
        shift_id=0,
        seed=seed
    )


def run_random(fn: Callable, dims: int, budget: int, seed: int) -> BenchmarkResult:
    """Run Random Search."""
    rng = random.Random(seed)
    best = float('inf')
    
    start = time.perf_counter()
    
    for _ in range(budget):
        params = {f"x{i}": rng.random() for i in range(dims)}
        value = fn(params)
        if value < best:
            best = value
    
    elapsed_ms = (time.perf_counter() - start) * 1000
    
    return BenchmarkResult(
        optimizer="Random",
        benchmark="",
        best_value=best,
        evaluations=budget,
        wall_time_ms=elapsed_ms,
        shift_id=0,
        seed=seed
    )


# =============================================================================
# Probe Quality Testing (Direct Python comparison)
# =============================================================================

def get_primes(n):
    """Generate first n primes."""
    if n == 0:
        return np.array([], dtype=int)
    limit = max(20, int(n * (np.log(n + 1) + np.log(np.log(n + 2)))) + 20)
    is_prime = np.ones(limit, dtype=bool)
    is_prime[:2] = False
    for i in range(2, int(limit**0.5) + 1):
        if is_prime[i]:
            is_prime[i*i::i] = False
    return np.nonzero(is_prime)[0][:n]


def probe_legacy(n_samples: int, dims: int) -> np.ndarray:
    """Legacy p/1000 implementation (for A/B comparison)."""
    primes = get_primes(n_samples)
    candidates = []
    base_positions = (primes / 1000.0) % 1.0
    
    for i in range(n_samples):
        point = []
        pos = base_positions[i]
        for d in range(dims):
            dim_offset = (d + 1) * 0.618033988749895
            val = (pos + dim_offset * (i / float(n_samples))) % 1.0
            point.append(val)
        candidates.append(point)
    return np.array(candidates)


def probe_new(n_samples: int, dims: int) -> np.ndarray:
    """New PrimeSqrtSlopesRotProbe."""
    primes = get_primes(210 + dims)
    prime_offset = 50
    rot_offset = 200
    rot_alpha = np.sqrt(2) - 1
    
    candidates = []
    for i in range(n_samples):
        point = []
        for d in range(dims):
            slope = np.sqrt(primes[prime_offset + d])
            rotation = (primes[rot_offset + d] * rot_alpha) % 1.0
            val = ((i + 1) * slope + rotation) % 1.0
            point.append(val)
        candidates.append(point)
    return np.array(candidates)


def evaluate_probe_on_function(probe_points: np.ndarray, fn: Callable) -> float:
    """Evaluate probe quality by finding best on objective."""
    best = float('inf')
    for x in probe_points:
        value = fn(x)
        if value < best:
            best = value
    return best


# =============================================================================
# Main Benchmark
# =============================================================================

def run_full_benchmark(
    budgets: List[int] = [32, 64, 128],
    n_shifts: int = 10,
    n_seeds: int = 3,
    master_seed: int = 20251214
) -> Dict:
    """Run comprehensive benchmark with torus shifts."""
    
    rng = np.random.default_rng(master_seed)
    results = []
    
    print("=" * 70)
    print("COMPREHENSIVE PROBE BENCHMARK")
    print("=" * 70)
    print(f"Budgets: {budgets}")
    print(f"Torus shifts: {n_shifts}")
    print(f"Seeds per shift: {n_seeds}")
    print(f"ArqonHPO available: {HAS_ARQON}")
    print(f"Optuna available: {HAS_OPTUNA}")
    print("=" * 70)
    
    for bench_name, bench_config in BENCHMARKS.items():
        fn_base = bench_config["fn"]
        dims = bench_config["dims"]
        
        print(f"\n>>> {bench_name} (dims={dims})")
        
        for budget in budgets:
            print(f"  Budget={budget}:")
            
            for shift_id in range(n_shifts):
                # Random torus shift
                shift = rng.random(dims)
                
                for seed in range(n_seeds):
                    actual_seed = master_seed + shift_id * 1000 + seed
                    
                    # Create shifted objective
                    fn = make_shifted_objective(fn_base, shift)
                    
                    # --- ArqonHPO (New Probe) ---
                    if HAS_ARQON:
                        r = run_arqonhpo(fn, dims, budget, actual_seed)
                        r.benchmark = bench_name
                        r.shift_id = shift_id
                        results.append(r)
                    
                    # --- Optuna ---
                    if HAS_OPTUNA:
                        r = run_optuna(fn, dims, budget, actual_seed)
                        r.benchmark = bench_name
                        r.shift_id = shift_id
                        results.append(r)
                    
                    # --- Random ---
                    r = run_random(fn, dims, budget, actual_seed)
                    r.benchmark = bench_name
                    r.shift_id = shift_id
                    results.append(r)
                    
                    # --- A/B Probe Test (direct comparison) ---
                    # Sample using legacy and new probe, evaluate on shifted function
                    probe_n = int(budget * 0.2)
                    legacy_pts = probe_legacy(probe_n, dims)
                    new_pts = probe_new(probe_n, dims)
                    
                    # Apply shift to probe points
                    legacy_pts_shifted = (legacy_pts + shift) % 1.0
                    new_pts_shifted = (new_pts + shift) % 1.0
                    
                    legacy_best = evaluate_probe_on_function(legacy_pts_shifted, fn_base)
                    new_best = evaluate_probe_on_function(new_pts_shifted, fn_base)
                    
                    results.append(BenchmarkResult(
                        optimizer="Probe (Legacy)",
                        benchmark=bench_name,
                        best_value=legacy_best,
                        evaluations=probe_n,
                        wall_time_ms=0,
                        shift_id=shift_id,
                        seed=seed
                    ))
                    results.append(BenchmarkResult(
                        optimizer="Probe (New)",
                        benchmark=bench_name,
                        best_value=new_best,
                        evaluations=probe_n,
                        wall_time_ms=0,
                        shift_id=shift_id,
                        seed=seed
                    ))
            
            print(f"    ✓ Completed {n_shifts * n_seeds} runs per optimizer")
    
    return results


def summarize_results(results: List[BenchmarkResult]) -> str:
    """Generate summary report."""
    
    lines = [
        "# Probe Upgrade Benchmark Results",
        "",
        f"**Date**: 2025-12-14",
        f"**Branch**: experiment/probe-upgrade",
        "",
        "## Summary Table (Mean Best Value ± SE)",
        "",
    ]
    
    # Group by benchmark and optimizer
    from collections import defaultdict
    grouped = defaultdict(lambda: defaultdict(list))
    
    for r in results:
        grouped[r.benchmark][r.optimizer].append(r.best_value)
    
    # Header
    optimizers = sorted(set(r.optimizer for r in results))
    header = "| Benchmark | " + " | ".join(optimizers) + " |"
    sep = "|-----------|" + "|".join(["----------"] * len(optimizers)) + "|"
    lines.extend([header, sep])
    
    # Data rows
    for bench_name in sorted(grouped.keys()):
        row = f"| {bench_name} |"
        for opt in optimizers:
            values = grouped[bench_name].get(opt, [])
            if values:
                mean_val = np.mean(values)
                se_val = np.std(values) / np.sqrt(len(values))
                row += f" {mean_val:.4f}±{se_val:.4f} |"
            else:
                row += " N/A |"
        lines.append(row)
    
    lines.extend([
        "",
        "## A/B Probe Comparison",
        "",
    ])
    
    # Calculate A/B wins
    legacy_wins = 0
    new_wins = 0
    ties = 0
    
    for bench_name in grouped.keys():
        legacy_vals = grouped[bench_name].get("Probe (Legacy)", [])
        new_vals = grouped[bench_name].get("Probe (New)", [])
        
        for l, n in zip(legacy_vals, new_vals):
            if l < n:
                legacy_wins += 1
            elif n < l:
                new_wins += 1
            else:
                ties += 1
    
    total = legacy_wins + new_wins + ties
    lines.extend([
        f"- **New Probe Wins**: {new_wins}/{total} ({100*new_wins/total:.1f}%)",
        f"- **Legacy Probe Wins**: {legacy_wins}/{total} ({100*legacy_wins/total:.1f}%)",
        f"- **Ties**: {ties}/{total}",
        "",
    ])
    
    if new_wins > legacy_wins:
        lines.append("**✅ NEW PROBE IS SUPERIOR**")
    elif legacy_wins > new_wins:
        lines.append("**⚠️ LEGACY PROBE PERFORMS BETTER (unexpected)**")
    else:
        lines.append("**➖ RESULTS ARE MIXED**")
    
    # Timing comparison
    lines.extend([
        "",
        "## Execution Time Comparison (mean ms)",
        "",
    ])
    
    time_data = defaultdict(list)
    for r in results:
        if r.wall_time_ms > 0:
            time_data[r.optimizer].append(r.wall_time_ms)
    
    for opt in sorted(time_data.keys()):
        mean_time = np.mean(time_data[opt])
        lines.append(f"- **{opt}**: {mean_time:.2f}ms")
    
    return "\n".join(lines)


if __name__ == "__main__":
    import os
    
    # Run benchmark
    results = run_full_benchmark(
        budgets=[32, 64, 128],
        n_shifts=10,
        n_seeds=3
    )
    
    # Generate report
    report = summarize_results(results)
    
    # Save
    output_path = "benchmarks/PROBE_UPGRADE_BENCHMARK.md"
    with open(output_path, "w") as f:
        f.write(report)
    
    print("\n" + "=" * 70)
    print("BENCHMARK COMPLETE")
    print("=" * 70)
    print(report)
    print(f"\nSaved to: {output_path}")
