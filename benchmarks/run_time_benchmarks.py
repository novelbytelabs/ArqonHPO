#!/usr/bin/env python3
"""
ArqonHPO Time-Bounded Benchmark Suite
Comparing "Best Value Found" within a fixed WALL-CLOCK budget.
Tests: ArqonHPO vs Optuna vs Random
Zones: Speed (0ms), Crossover (1ms), Brain (100ms)
"""

import json
import time
import random
import statistics
from dataclasses import dataclass, field
from typing import Callable, Dict, List, Tuple
import matplotlib.pyplot as plt
import numpy as np

# Imports
try:
    from arqonhpo import ArqonSolver
    HAS_ARQON = True
except ImportError:
    HAS_ARQON = False
    print("⚠️  ArqonHPO not installed")

try:
    import optuna
    optuna.logging.set_verbosity(optuna.logging.WARNING)
    HAS_OPTUNA = True
except ImportError:
    HAS_OPTUNA = False
    print("⚠️  Optuna not installed")


# =============================================================================
# HARD TEST FUNCTIONS (50D)
# =============================================================================

def rosenbrock_50d(params: Dict[str, float]) -> float:
    """Rosenbrock 50D: Smooth, valley-like, hard to converge."""
    vals = list(params.values())
    total = 0.0
    for i in range(len(vals) - 1):
        total += 100 * (vals[i+1] - vals[i]**2)**2 + (1 - vals[i])**2
    return total

def rastrigin_50d(params: Dict[str, float]) -> float:
    """Rastrigin 50D: Highly multimodal."""
    A = 10
    n = len(params)
    vals = list(params.values())
    return A * n + sum(v**2 - A * np.cos(2 * np.pi * v) for v in vals)

BENCHMARKS = {
    "Rosenbrock-50D": {"fn": rosenbrock_50d, "dims": 50, "bounds": (-2.0, 2.0), "optimal": 0.0},
    "Rastrigin-50D": {"fn": rastrigin_50d, "dims": 50, "bounds": (-5.12, 5.12), "optimal": 0.0},
}

# =============================================================================
# RUNNER CONFIG
# =============================================================================

@dataclass
class TimeResult:
    optimizer: str
    benchmark: str
    latency_name: str
    latency_sec: float
    time_limit: float
    best_value: float
    evaluations: int
    trajectory: List[Tuple[float, float]] = field(default_factory=list) # (time, best_value)

ZONES = {
    "🚀 Speed (0ms)": 0.0,
    "⚖️ Crossover (1ms)": 0.001,
    "🧠 Brain (100ms)": 0.1,
}

# =============================================================================
# OPTIMIZERS
# =============================================================================

def run_arqon_timed(fn: Callable, dims: int, bounds: Tuple[float, float], 
                    time_limit: float, latency: float, seed: int) -> TimeResult:
    config = {
        "seed": seed,
        "budget": 1_000_000, # Effectively infinite
        "bounds": {f"x{i}": {"min": bounds[0], "max": bounds[1]} for i in range(dims)},
        "probe_ratio": 0.2
    }
    
    solver = ArqonSolver(json.dumps(config))
    best = float('inf')
    traj = []
    evals = 0
    
    start_time = time.perf_counter()
    
    while True:
        now = time.perf_counter()
        if (now - start_time) > time_limit:
            break
            
        candidates = solver.ask()
        if candidates is None:
            break
            
        results = []
        for params in candidates:
            # Check time again inside batch
            loop_now = time.perf_counter()
            if (loop_now - start_time) > time_limit:
                break

            # Simulate work
            if latency > 0:
                time.sleep(latency)
            
            value = fn(params)
            
            if value < best:
                best = value
                traj.append((loop_now - start_time, best))
            
            evals += 1
            results.append({"eval_id": evals, "params": params, "value": value, "cost": 1.0})
            
        solver.tell(json.dumps(results))
        
    return TimeResult("ArqonHPO", "", "", latency, time_limit, best, evals, traj)


def run_optuna_timed(fn: Callable, dims: int, bounds: Tuple[float, float], 
                     time_limit: float, latency: float, seed: int) -> TimeResult:
    
    best = float('inf')
    traj = []
    evals = 0
    start_time = time.perf_counter()
    
    def objective(trial):
        nonlocal best, evals
        
        # Check timeout
        if (time.perf_counter() - start_time) > time_limit:
            raise optuna.exceptions.TrialPruned()
            
        params = {f"x{i}": trial.suggest_float(f"x{i}", bounds[0], bounds[1]) for i in range(dims)}
        
        # Simulate work
        if latency > 0:
            time.sleep(latency)
            
        value = fn(params)
        
        if value < best:
            best = value
            traj.append((time.perf_counter() - start_time, best))
        
        evals += 1
        return value

    sampler = optuna.samplers.TPESampler(seed=seed)
    study = optuna.create_study(direction="minimize", sampler=sampler)
    
    try:
        study.optimize(objective, timeout=time_limit, n_trials=1_000_000, show_progress_bar=False)
    except Exception:
        pass # Timeout handling
        
    return TimeResult("Optuna", "", "", latency, time_limit, best, evals, traj)


def run_random_timed(fn: Callable, dims: int, bounds: Tuple[float, float], 
                     time_limit: float, latency: float, seed: int) -> TimeResult:
    
    rng = random.Random(seed)
    best = float('inf')
    traj = []
    evals = 0
    start_time = time.perf_counter()
    
    while (time.perf_counter() - start_time) < time_limit:
        params = {f"x{i}": rng.uniform(bounds[0], bounds[1]) for i in range(dims)}
        
        if latency > 0:
            time.sleep(latency)
            
        value = fn(params)
        
        if value < best:
            best = value
            traj.append((time.perf_counter() - start_time, best))
        
        evals += 1
        
    return TimeResult("Random", "", "", latency, time_limit, best, evals, traj)


# =============================================================================
# MAIN RUNNER
# =============================================================================

def run_battle(time_budget: float = 3.0, seeds: int = 3):
    results = []
    
    print(f"🥊 BATTLE START: {time_budget}s Time Budget\n")
    
    for bench_name, bench_cfg in BENCHMARKS.items():
        print(f"=== {bench_name} ===")
        
        for zone_name, latency in ZONES.items():
            print(f"  Mapping {zone_name}...")
            
            for seed in range(seeds):
                # Arqon
                if HAS_ARQON:
                    r = run_arqon_timed(bench_cfg["fn"], bench_cfg["dims"], bench_cfg["bounds"], 
                                      time_budget, latency, seed)
                    r.benchmark = bench_name
                    r.latency_name = zone_name
                    results.append(r)
                
                # Optuna
                if HAS_OPTUNA:
                    r = run_optuna_timed(bench_cfg["fn"], bench_cfg["dims"], bench_cfg["bounds"], 
                                       time_budget, latency, seed)
                    r.benchmark = bench_name
                    r.latency_name = zone_name
                    results.append(r)
                
                # Random
                r = run_random_timed(bench_cfg["fn"], bench_cfg["dims"], bench_cfg["bounds"], 
                                   time_budget, latency, seed)
                r.benchmark = bench_name
                r.latency_name = zone_name
                results.append(r)
    
    return results

# =============================================================================
# VISUALIZATION
# =============================================================================

def plot_zone_map(results: List[TimeResult], output_dir: str):
    """Plot Best Value (y) vs Evaluation Count (x bar) for each Zone."""
    
    # Aggregate
    agg = {} # (benchmark, zone, optimizer) -> (mean_best, mean_evals)
    
    for r in results:
        key = (r.benchmark, r.latency_name, r.optimizer)
        if key not in agg:
            agg[key] = {"vals": [], "evals": []}
        agg[key]["vals"].append(r.best_value)
        agg[key]["evals"].append(r.evaluations)
        
    summary = {}
    for k, v in agg.items():
        summary[k] = (statistics.mean(v["vals"]), statistics.mean(v["evals"]))
        
    benchmarks = list(BENCHMARKS.keys())
    zones = list(ZONES.keys())
    
    fig, axes = plt.subplots(len(benchmarks), len(zones), figsize=(18, 10), constrained_layout=True)
    
    colors = {"ArqonHPO": "#2196F3", "Optuna": "#4CAF50", "Random": "#9E9E9E"}
    
    for row, bench in enumerate(benchmarks):
        for col, zone in enumerate(zones):
            ax = axes[row][col] if len(benchmarks) > 1 else axes[col]
            
            opts = ["ArqonHPO", "Optuna", "Random"]
            
            # Get data
            vals = [summary.get((bench, zone, o), (0,0))[0] for o in opts]
            evals = [summary.get((bench, zone, o), (0,0))[1] for o in opts]
            
            # Left axis: Best Value (Bar)
            x = np.arange(len(opts))
            bars = ax.bar(x, vals, width=0.4, color=[colors[o] for o in opts], alpha=0.7, label="Best Value")
            
            # Add eval counts on top
            for i, rect in enumerate(bars):
                height = rect.get_height()
                ax.text(rect.get_x() + rect.get_width()/2., height,
                        f'{evals[i]:.0f} evals',
                        ha='center', va='bottom', rotation=0, fontsize=9, fontweight='bold')

            ax.set_title(f"{bench}\n{zone}", fontsize=11)
            ax.set_xticks(x)
            ax.set_xticklabels(opts)
            if col == 0:
                ax.set_ylabel("Best Value Found (Lower is Better)")
            
            ax.set_yscale('log')
            ax.grid(True, alpha=0.3, axis='y')
            
            # Highlight winner
            best_idx = np.argmin(vals)
            ax.text(0.95, 0.95, f"Winner:\n{opts[best_idx]}", 
                   transform=ax.transAxes, ha='right', va='top', 
                   bbox=dict(boxstyle='round', facecolor='white', alpha=0.8))

    plt.suptitle(f"ArqonHPO vs Optuna: Time-Bounded Battle (5s Budget)", fontsize=16)
    plt.savefig(f"{output_dir}/time_bounded_zones.png", dpi=150)
    print(f"Saved {output_dir}/time_bounded_zones.png")


if __name__ == "__main__":
    import os
    os.makedirs("benchmarks", exist_ok=True)
    
    results = run_battle(time_budget=5.0, seeds=3)
    plot_zone_map(results, "benchmarks")
    
    # Text Report
    print("\n=== SUMMARY ===")
    agg = {}
    for r in results:
        key = (r.benchmark, r.latency_name, r.optimizer)
        if key not in agg: agg[key] = []
        agg[key].append(r.best_value)
        
    for k, vals in agg.items():
        bench, zone, opt = k
        mean = statistics.mean(vals)
        print(f"{bench:15s} | {zone:20s} | {opt:10s} | {mean:.4f}")
