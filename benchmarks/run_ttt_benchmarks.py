#!/usr/bin/env python3
"""
TIME-TO-TARGET BENCHMARK
========================

The REAL metric: How long does it take to reach a quality threshold?

ArqonHPO is 297x faster per trial. If we need even 100x more trials,
we're still 3x faster in wall-clock time.

This benchmark measures:
1. Time to reach target quality (e.g., best < 0.1)
2. Number of evaluations to reach target
3. Whether target was reached at all
"""

import json
import time
import numpy as np
from typing import Dict, Callable, Tuple, Optional
from dataclasses import dataclass

# Import ArqonHPO
try:
    from arqonhpo import ArqonSolver
    HAS_ARQON = True
except ImportError:
    HAS_ARQON = False
    print("ERROR: ArqonHPO not installed")
    exit(1)

# Import Optuna
try:
    import optuna
    optuna.logging.set_verbosity(optuna.logging.WARNING)
    HAS_OPTUNA = True
except ImportError:
    HAS_OPTUNA = False
    print("ERROR: Optuna not installed")
    exit(1)


# =============================================================================
# TEST FUNCTIONS
# =============================================================================

def sphere(params: Dict[str, float]) -> float:
    return sum(v**2 for v in params.values())

def rosenbrock(params: Dict[str, float]) -> float:
    vals = list(params.values())
    total = 0.0
    for i in range(len(vals) - 1):
        total += 100 * (vals[i+1] - vals[i]**2)**2 + (1 - vals[i])**2
    return total

def rastrigin(params: Dict[str, float]) -> float:
    A = 10
    n = len(params)
    vals = list(params.values())
    return A * n + sum(v**2 - A * np.cos(2 * np.pi * v) for v in vals)

def ackley(params: Dict[str, float]) -> float:
    vals = np.array(list(params.values()))
    n = len(vals)
    sum1 = np.sum(vals**2)
    sum2 = np.sum(np.cos(2 * np.pi * vals))
    return -20 * np.exp(-0.2 * np.sqrt(sum1 / n)) - np.exp(sum2 / n) + 20 + np.e


# =============================================================================
# RESULTS
# =============================================================================

@dataclass
class TTTResult:
    """Time-to-Target result."""
    optimizer: str
    target_reached: bool
    time_to_target: float  # seconds, -1 if not reached
    evals_to_target: int   # -1 if not reached
    final_best: float
    total_time: float
    total_evals: int


# =============================================================================
# ARQONHPO RUNNER
# =============================================================================

def run_arqon_ttt(
    fn: Callable,
    dims: int,
    bounds: Tuple[float, float],
    target: float,
    max_budget: int,
    seed: int
) -> TTTResult:
    """Run ArqonHPO until target is reached or budget exhausted."""
    
    param_bounds = {f"x{i}": {"min": bounds[0], "max": bounds[1]} for i in range(dims)}
    config = {
        "seed": seed,
        "budget": max_budget,
        "probe_ratio": 0.2,
        "bounds": param_bounds
    }
    
    solver = ArqonSolver(json.dumps(config))
    
    best_value = float('inf')
    total_evals = 0
    target_reached = False
    time_to_target = -1.0
    evals_to_target = -1
    
    start_time = time.perf_counter()
    
    while True:
        batch = solver.ask()
        if batch is None:
            break
        
        results = []
        for params in batch:
            value = fn(params)
            total_evals += 1
            
            if value < best_value:
                best_value = value
            
            if not target_reached and best_value <= target:
                target_reached = True
                time_to_target = time.perf_counter() - start_time
                evals_to_target = total_evals
            
            results.append({
                "eval_id": total_evals,
                "params": params,
                "value": value,
                "cost": 1.0
            })
        
        solver.tell(json.dumps(results))
    
    total_time = time.perf_counter() - start_time
    
    return TTTResult(
        optimizer="ArqonHPO",
        target_reached=target_reached,
        time_to_target=time_to_target,
        evals_to_target=evals_to_target,
        final_best=best_value,
        total_time=total_time,
        total_evals=total_evals
    )


# =============================================================================
# OPTUNA RUNNER  
# =============================================================================

def run_optuna_ttt(
    fn: Callable,
    dims: int,
    bounds: Tuple[float, float],
    target: float,
    max_budget: int,
    seed: int
) -> TTTResult:
    """Run Optuna-TPE until target is reached or budget exhausted."""
    
    best_value = float('inf')
    target_reached = False
    time_to_target = -1.0
    evals_to_target = -1
    total_evals = 0
    
    start_time = time.perf_counter()
    
    def objective(trial):
        nonlocal best_value, target_reached, time_to_target, evals_to_target, total_evals
        
        params = {f"x{i}": trial.suggest_float(f"x{i}", bounds[0], bounds[1]) 
                  for i in range(dims)}
        
        value = fn(params)
        total_evals += 1
        
        if value < best_value:
            best_value = value
        
        if not target_reached and best_value <= target:
            target_reached = True
            time_to_target = time.perf_counter() - start_time
            evals_to_target = total_evals
        
        return value
    
    sampler = optuna.samplers.TPESampler(seed=seed)
    study = optuna.create_study(sampler=sampler, direction="minimize")
    study.optimize(objective, n_trials=max_budget, show_progress_bar=False)
    
    total_time = time.perf_counter() - start_time
    
    return TTTResult(
        optimizer="Optuna-TPE",
        target_reached=target_reached,
        time_to_target=time_to_target,
        evals_to_target=evals_to_target,
        final_best=best_value,
        total_time=total_time,
        total_evals=total_evals
    )


# =============================================================================
# BENCHMARK SUITE
# =============================================================================

BENCHMARKS = {
    "Sphere-2D": {"fn": sphere, "dims": 2, "bounds": (-5.0, 5.0), "target": 0.01},
    "Sphere-5D": {"fn": sphere, "dims": 5, "bounds": (-5.0, 5.0), "target": 0.1},
    "Rosenbrock-2D": {"fn": rosenbrock, "dims": 2, "bounds": (-5.0, 10.0), "target": 1.0},
    "Rastrigin-2D": {"fn": rastrigin, "dims": 2, "bounds": (-5.12, 5.12), "target": 1.0},
    "Ackley-2D": {"fn": ackley, "dims": 2, "bounds": (-5.0, 5.0), "target": 0.5},
}


def run_ttt_benchmark(max_budget: int = 500, num_runs: int = 5):
    """Run time-to-target benchmark."""
    
    print("=" * 70)
    print("TIME-TO-TARGET BENCHMARK")
    print("The ONLY metric that matters: How fast do we reach the target?")
    print("=" * 70)
    print()
    
    all_results = {}
    
    for bench_name, config in BENCHMARKS.items():
        print(f"\n{'=' * 60}")
        print(f"📊 {bench_name} | Target: {config['target']}")
        print("=" * 60)
        
        arqon_results = []
        optuna_results = []
        
        for run in range(num_runs):
            seed = 42 + run
            
            # Run ArqonHPO
            arqon = run_arqon_ttt(
                config["fn"], config["dims"], config["bounds"],
                config["target"], max_budget, seed
            )
            arqon_results.append(arqon)
            
            # Run Optuna
            optuna_res = run_optuna_ttt(
                config["fn"], config["dims"], config["bounds"],
                config["target"], max_budget, seed
            )
            optuna_results.append(optuna_res)
            
            print(f"  Run {run+1}:")
            print(f"    ArqonHPO: ", end="")
            if arqon.target_reached:
                print(f"✅ {arqon.time_to_target*1000:.1f}ms ({arqon.evals_to_target} evals)")
            else:
                print(f"❌ not reached (best: {arqon.final_best:.4f})")
            
            print(f"    Optuna:   ", end="")
            if optuna_res.target_reached:
                print(f"✅ {optuna_res.time_to_target*1000:.1f}ms ({optuna_res.evals_to_target} evals)")
            else:
                print(f"❌ not reached (best: {optuna_res.final_best:.4f})")
        
        all_results[bench_name] = {
            "arqon": arqon_results,
            "optuna": optuna_results
        }
    
    # Summary
    print("\n" + "=" * 70)
    print("📈 TIME-TO-TARGET SUMMARY")
    print("=" * 70)
    print()
    print(f"{'Benchmark':<20} | {'ArqonHPO':<20} | {'Optuna-TPE':<20} | {'Speedup':>10}")
    print("-" * 75)
    
    for bench_name, results in all_results.items():
        arqon_times = [r.time_to_target for r in results["arqon"] if r.target_reached]
        optuna_times = [r.time_to_target for r in results["optuna"] if r.target_reached]
        
        arqon_rate = len(arqon_times) / num_runs * 100
        optuna_rate = len(optuna_times) / num_runs * 100
        
        if arqon_times and optuna_times:
            arqon_avg = sum(arqon_times) / len(arqon_times) * 1000
            optuna_avg = sum(optuna_times) / len(optuna_times) * 1000
            speedup = optuna_avg / arqon_avg
            
            arqon_str = f"{arqon_avg:.1f}ms ({arqon_rate:.0f}%)"
            optuna_str = f"{optuna_avg:.1f}ms ({optuna_rate:.0f}%)"
            speedup_str = f"{speedup:.1f}x"
        elif arqon_times:
            arqon_avg = sum(arqon_times) / len(arqon_times) * 1000
            arqon_str = f"{arqon_avg:.1f}ms ({arqon_rate:.0f}%)"
            optuna_str = f"N/A ({optuna_rate:.0f}%)"
            speedup_str = "ArqonHPO wins"
        elif optuna_times:
            optuna_avg = sum(optuna_times) / len(optuna_times) * 1000
            arqon_str = f"N/A ({arqon_rate:.0f}%)"
            optuna_str = f"{optuna_avg:.1f}ms ({optuna_rate:.0f}%)"
            speedup_str = "Optuna wins"
        else:
            arqon_str = f"N/A ({arqon_rate:.0f}%)"
            optuna_str = f"N/A ({optuna_rate:.0f}%)"
            speedup_str = "N/A"
        
        print(f"{bench_name:<20} | {arqon_str:<20} | {optuna_str:<20} | {speedup_str:>10}")
    
    print("-" * 75)
    print()
    
    # Calculate overall winner
    arqon_wins = 0
    optuna_wins = 0
    total_arqon_speedup = []
    
    for bench_name, results in all_results.items():
        arqon_times = [r.time_to_target for r in results["arqon"] if r.target_reached]
        optuna_times = [r.time_to_target for r in results["optuna"] if r.target_reached]
        
        if arqon_times and optuna_times:
            arqon_avg = sum(arqon_times) / len(arqon_times)
            optuna_avg = sum(optuna_times) / len(optuna_times)
            
            if arqon_avg < optuna_avg:
                arqon_wins += 1
                total_arqon_speedup.append(optuna_avg / arqon_avg)
            else:
                optuna_wins += 1
    
    print(f"🏆 WINNER: ", end="")
    if arqon_wins > optuna_wins:
        avg_speedup = sum(total_arqon_speedup) / len(total_arqon_speedup) if total_arqon_speedup else 0
        print(f"ArqonHPO ({arqon_wins}/{arqon_wins + optuna_wins} benchmarks)")
        print(f"   Average speedup: {avg_speedup:.1f}x faster to reach target")
    elif optuna_wins > arqon_wins:
        print(f"Optuna-TPE ({optuna_wins}/{arqon_wins + optuna_wins} benchmarks)")
    else:
        print("TIE")
    
    return all_results


if __name__ == "__main__":
    results = run_ttt_benchmark(max_budget=500, num_runs=5)
