#!/usr/bin/env python3
"""
Ablation Benchmark: Isolate Probe vs Refinement Contributions

This benchmark answers the question: "Where does the 2-3× quality gap vs Optuna come from?"

Tests:
A) Hold refinement constant, swap probes
B) Hold probe constant, swap refinement

Run: conda run -n helios-gpu-118 python benchmarks/run_ablation_benchmark.py
"""

import json
import time
import random
from dataclasses import dataclass, field
from typing import Callable, Dict, List, Tuple
import numpy as np
from collections import defaultdict

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
# Probe Implementations (Unit Cube)
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


def probe_legacy(n_samples: int, dims: int, seed: int = 0) -> np.ndarray:
    """Legacy p/1000 probe."""
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


def probe_new(n_samples: int, dims: int, seed: int = 0) -> np.ndarray:
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


def probe_random(n_samples: int, dims: int, seed: int = 0) -> np.ndarray:
    """Random uniform probe."""
    rng = np.random.default_rng(seed)
    return rng.random((n_samples, dims))


PROBES = {
    "Legacy": probe_legacy,
    "New": probe_new,
    "Random": probe_random,
}


# =============================================================================
# Refinement Implementations (Local Search on Unit Cube)
# =============================================================================

def refine_none(init_points: np.ndarray, fn: Callable, budget: int, seed: int) -> float:
    """No refinement - just return best from initial points."""
    best = float('inf')
    for x in init_points:
        val = fn(x)
        if val < best:
            best = val
    return best


def refine_nelder_mead_simple(init_points: np.ndarray, fn: Callable, budget: int, seed: int) -> float:
    """Simple Nelder-Mead from best initial point."""
    from scipy.optimize import minimize
    
    # Find best initial point
    values = [fn(x) for x in init_points]
    best_idx = np.argmin(values)
    x0 = init_points[best_idx]
    best = values[best_idx]
    
    # Run NM with remaining budget
    remaining = budget - len(init_points)
    if remaining <= 0:
        return best
    
    evals = [0]  # Mutable counter
    
    def objective(x):
        evals[0] += 1
        if evals[0] > remaining:
            raise StopIteration()
        x_clipped = np.clip(x, 0, 1)
        return fn(x_clipped)
    
    try:
        result = minimize(objective, x0, method='Nelder-Mead', 
                         options={'maxfev': remaining, 'xatol': 1e-8, 'fatol': 1e-8})
        if result.fun < best:
            best = result.fun
    except StopIteration:
        pass
    
    return best


def refine_optuna_tpe(init_points: np.ndarray, fn: Callable, budget: int, seed: int) -> float:
    """Optuna TPE refinement (seeded with initial points)."""
    dims = init_points.shape[1]
    
    # Evaluate initial points first
    init_values = [fn(x) for x in init_points]
    best = min(init_values)
    remaining = budget - len(init_points)
    
    if remaining <= 0 or not HAS_OPTUNA:
        return best
    
    def objective(trial):
        nonlocal best
        x = np.array([trial.suggest_float(f"x{i}", 0.0, 1.0) for i in range(dims)])
        val = fn(x)
        if val < best:
            best = val
        return val
    
    sampler = optuna.samplers.TPESampler(seed=seed)
    study = optuna.create_study(direction="minimize", sampler=sampler)
    
    # Add initial points as completed trials
    for x, val in zip(init_points, init_values):
        study.add_trial(
            optuna.trial.create_trial(
                params={f"x{i}": x[i] for i in range(dims)},
                distributions={f"x{i}": optuna.distributions.FloatDistribution(0.0, 1.0) for i in range(dims)},
                values=[val],
            )
        )
    
    study.optimize(objective, n_trials=remaining, show_progress_bar=False)
    return min(best, study.best_value)


def refine_random(init_points: np.ndarray, fn: Callable, budget: int, seed: int) -> float:
    """Random search refinement."""
    dims = init_points.shape[1]
    rng = np.random.default_rng(seed)
    
    best = float('inf')
    for x in init_points:
        val = fn(x)
        if val < best:
            best = val
    
    remaining = budget - len(init_points)
    for _ in range(remaining):
        x = rng.random(dims)
        val = fn(x)
        if val < best:
            best = val
    
    return best


REFINERS = {
    "None": refine_none,
    "NM": refine_nelder_mead_simple,
    "TPE": refine_optuna_tpe,
    "Random": refine_random,
}


# =============================================================================
# Benchmark Functions
# =============================================================================

def rosenbrock(x: np.ndarray) -> float:
    x = (x * 2 - 1) * 2.048
    return float(np.sum(100 * (x[1:] - x[:-1]**2)**2 + (1 - x[:-1])**2))


def sphere(x: np.ndarray) -> float:
    x = (x * 2 - 1) * 5.0
    return float(np.sum(x**2))


def rastrigin(x: np.ndarray) -> float:
    x = (x * 2 - 1) * 5.12
    d = len(x)
    return float(10 * d + np.sum(x**2 - 10 * np.cos(2 * np.pi * x)))


FUNCTIONS = {
    "Sphere (5D)": (sphere, 5),
    "Rosenbrock (5D)": (rosenbrock, 5),
    "Rastrigin (5D)": (rastrigin, 5),
}


# =============================================================================
# Ablation Tests
# =============================================================================

@dataclass
class AblationResult:
    test_type: str
    probe: str
    refiner: str
    function: str
    best_value: float
    seed: int


def run_ablation_a(budget: int = 100, probe_ratio: float = 0.2, n_seeds: int = 10) -> List[AblationResult]:
    """A) Hold refinement constant, swap probes."""
    results = []
    probe_budget = int(budget * probe_ratio)
    
    print("\n=== ABLATION A: Hold Refinement (NM), Swap Probes ===\n")
    
    for fn_name, (fn, dims) in FUNCTIONS.items():
        print(f"  {fn_name}:")
        for probe_name, probe_fn in PROBES.items():
            values = []
            for seed in range(n_seeds):
                # Get probe points
                probe_points = probe_fn(probe_budget, dims, seed)
                # Refine with NM
                best = refine_nelder_mead_simple(probe_points, fn, budget, seed)
                values.append(best)
                results.append(AblationResult(
                    test_type="A",
                    probe=probe_name,
                    refiner="NM",
                    function=fn_name,
                    best_value=best,
                    seed=seed
                ))
            print(f"    {probe_name:8s}: {np.mean(values):10.4f} ± {np.std(values):8.4f}")
    
    return results


def run_ablation_b(budget: int = 100, probe_ratio: float = 0.2, n_seeds: int = 10) -> List[AblationResult]:
    """B) Hold probe constant (New), swap refinement."""
    results = []
    probe_budget = int(budget * probe_ratio)
    
    print("\n=== ABLATION B: Hold Probe (New), Swap Refinement ===\n")
    
    for fn_name, (fn, dims) in FUNCTIONS.items():
        print(f"  {fn_name}:")
        for refiner_name, refiner_fn in REFINERS.items():
            if refiner_name == "TPE" and not HAS_OPTUNA:
                continue
            values = []
            for seed in range(n_seeds):
                # Get probe points (always New)
                probe_points = probe_new(probe_budget, dims, seed)
                # Refine with different strategies
                best = refiner_fn(probe_points, fn, budget, seed)
                values.append(best)
                results.append(AblationResult(
                    test_type="B",
                    probe="New",
                    refiner=refiner_name,
                    function=fn_name,
                    best_value=best,
                    seed=seed
                ))
            print(f"    {refiner_name:8s}: {np.mean(values):10.4f} ± {np.std(values):8.4f}")
    
    return results


def analyze_results(results: List[AblationResult]) -> str:
    """Generate analysis report."""
    lines = [
        "# Ablation Benchmark Results",
        "",
        f"**Date**: 2025-12-14",
        "",
    ]
    
    # Group results
    by_test = defaultdict(list)
    for r in results:
        by_test[r.test_type].append(r)
    
    # Ablation A
    lines.append("## A) Hold Refinement (NM), Swap Probes")
    lines.append("")
    lines.append("| Function | Legacy Probe | New Probe | Random Probe | Best |")
    lines.append("|----------|-------------|-----------|--------------|------|")
    
    for fn_name in FUNCTIONS.keys():
        row = f"| {fn_name} |"
        best_val = float('inf')
        best_probe = ""
        for probe_name in ["Legacy", "New", "Random"]:
            vals = [r.best_value for r in by_test["A"] 
                    if r.function == fn_name and r.probe == probe_name]
            if vals:
                mean = np.mean(vals)
                row += f" {mean:.4f} |"
                if mean < best_val:
                    best_val = mean
                    best_probe = probe_name
            else:
                row += " N/A |"
        row += f" {best_probe} |"
        lines.append(row)
    
    lines.append("")
    
    # Ablation B
    lines.append("## B) Hold Probe (New), Swap Refinement")
    lines.append("")
    lines.append("| Function | None | NM | TPE | Random | Best |")
    lines.append("|----------|------|----|----|--------|------|")
    
    for fn_name in FUNCTIONS.keys():
        row = f"| {fn_name} |"
        best_val = float('inf')
        best_refiner = ""
        for refiner_name in ["None", "NM", "TPE", "Random"]:
            vals = [r.best_value for r in by_test["B"] 
                    if r.function == fn_name and r.refiner == refiner_name]
            if vals:
                mean = np.mean(vals)
                row += f" {mean:.4f} |"
                if mean < best_val:
                    best_val = mean
                    best_refiner = refiner_name
            else:
                row += " N/A |"
        row += f" {best_refiner} |"
        lines.append(row)
    
    lines.append("")
    
    # Analysis
    lines.append("## Key Findings")
    lines.append("")
    
    # Compute probe contribution
    probe_improvements = []
    for fn_name in FUNCTIONS.keys():
        legacy = np.mean([r.best_value for r in by_test["A"] 
                         if r.function == fn_name and r.probe == "Legacy"])
        new = np.mean([r.best_value for r in by_test["A"] 
                      if r.function == fn_name and r.probe == "New"])
        if legacy > 0:
            probe_improvements.append((legacy - new) / legacy * 100)
    
    avg_probe_improvement = np.mean(probe_improvements) if probe_improvements else 0
    lines.append(f"- **Probe contribution**: New probe is {avg_probe_improvement:.1f}% better than Legacy on average")
    
    # Compute refinement gap
    refine_gaps = []
    for fn_name in FUNCTIONS.keys():
        nm_vals = [r.best_value for r in by_test["B"] 
                   if r.function == fn_name and r.refiner == "NM"]
        tpe_vals = [r.best_value for r in by_test["B"] 
                    if r.function == fn_name and r.refiner == "TPE"]
        if nm_vals and tpe_vals:
            nm = np.mean(nm_vals)
            tpe = np.mean(tpe_vals)
            if tpe > 0:
                refine_gaps.append(nm / tpe)
    
    avg_refine_gap = np.mean(refine_gaps) if refine_gaps else 1.0
    lines.append(f"- **Refinement gap**: NM is {avg_refine_gap:.1f}× worse than TPE on average")
    lines.append("")
    
    if avg_refine_gap > 1.5:
        lines.append("**⚠️ CONCLUSION: The gap is primarily in REFINEMENT (NM vs TPE)**")
    elif avg_probe_improvement > 10:
        lines.append("**✅ CONCLUSION: The probe upgrade is working well**")
    else:
        lines.append("**➖ CONCLUSION: Mixed results, need more investigation**")
    
    return "\n".join(lines)


if __name__ == "__main__":
    print("=" * 70)
    print("ABLATION BENCHMARK: Isolate Probe vs Refinement Contributions")
    print("=" * 70)
    print(f"ArqonHPO available: {HAS_ARQON}")
    print(f"Optuna available: {HAS_OPTUNA}")
    
    # Run ablations
    results_a = run_ablation_a(budget=100, probe_ratio=0.2, n_seeds=10)
    results_b = run_ablation_b(budget=100, probe_ratio=0.2, n_seeds=10)
    
    # Analyze
    all_results = results_a + results_b
    report = analyze_results(all_results)
    
    # Save
    output_path = "benchmarks/ABLATION_BENCHMARK.md"
    with open(output_path, "w") as f:
        f.write(report)
    
    print("\n" + "=" * 70)
    print("RESULTS")
    print("=" * 70)
    print(report)
    print(f"\nSaved to: {output_path}")
