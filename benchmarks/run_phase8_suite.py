
import argparse
import json
import time
import hashlib
import csv
import sys
import os
import random
import numpy as np

# Adjust path to find bindings if not installed
sys.path.append(os.path.join(os.path.dirname(__file__), "../bindings/python"))

# Attempt imports
try:
    import arqonhpo
    HAS_ARQON = True
except ImportError:
    HAS_ARQON = False
    print("Warning: arqonhpo not found.")

try:
    import optuna
    HAS_OPTUNA = True
    optuna.logging.set_verbosity(optuna.logging.WARNING)
except ImportError:
    HAS_OPTUNA = False

from reporting.schema import *
import reporting.aggregate as agg
import reporting.plot as plotter

# --- Utilities ---

def deterministic_run_id(params):
    """Generate SHA256 run_id from params dict"""
    # Sort keys for stability
    s = json.dumps(params, sort_keys=True)
    return hashlib.sha256(s.encode()).hexdigest()[:16]

def get_shift_vec(dim, seed, shift_id, policy):
    """Deterministic shift vector generation"""
    # Use independent RNG state
    rng = random.Random(f"{seed}:{shift_id}")
    if policy == "smooth":
        # Smooth shift: [-2, 2] translation
        return [rng.uniform(-2.0, 2.0) for _ in range(dim)]
    elif policy == "torus":
        # Torus shift: [0, 1] random offset (modulo applies later)
        return [rng.uniform(0.0, 1.0) for _ in range(dim)]
    else:
        return [0.0] * dim

# --- Objectives ---

def sphere(x):
    return sum(xi*xi for xi in x)

def rosenbrock(x):
    val = 0.0
    for i in range(len(x)-1):
        val += 100.0 * (x[i+1] - x[i]**2)**2 + (1.0 - x[i])**2
    return val

def rastrigin(x):
    A = 10.0
    return A * len(x) + sum(xi**2 - A * np.cos(2 * np.pi * xi) for xi in x)

WORKLOADS = {
    "sphere_smooth_shift": sphere,
    "rosenbrock_smooth_shift": rosenbrock,
    "rastrigin_torus": rastrigin
}

class ObjectiveWrapper:
    def __init__(self, name, base_fn, shift_vec, policy, delay_ms, threshold):
        self.name = name
        self.base_fn = base_fn
        self.shift_vec = shift_vec
        self.policy = policy
        self.delay_ms = delay_ms
        self.threshold = threshold
        self.evals = [] # [(time, val)]
        self.start_time = None
        self.best_so_far = float('inf')
        self.hit_threshold = False
        
    def __call__(self, x_dict):
        # x_dict: {"x0": val, ...} or list?
        # Arqon ask returns dict list
        # Optuna suggest returns float
        
        if self.start_time is None:
            self.start_time = time.perf_counter()
            
        # Extract vector (assuming dict x0..xn or list)
        if isinstance(x_dict, dict):
            # Sort keys x0, x1...
            dim = len(self.shift_vec)
            vec = [x_dict[f"x{i}"] for i in range(dim)]
        else:
            vec = x_dict
            
        # Apply Shift
        shifted_vec = []
        for i, val in enumerate(vec):
            s = self.shift_vec[i]
            if self.policy == "torus":
                 # Modulo shift
                 shifted_vec.append((val + s) % 1.0) # Assuming [0,1] domain?
                 # Wait, Rastrigin is usually [-5.12, 5.12].
                 # If we assume normalized [0,1] input from optimizer...
                 # Arqon usually works in [0,1]? Or Config defines bounds.
                 # Phase 4 assumption: Arqon maps [0,1] -> Bounds.
                 # If we apply shift INSIDE objective, we assume input is in Bounds?
                 # Let's assume input is raw value from optimizer.
                 # If Arqon returns mapped value...
                 # Simpler: Apply shift to input.
                 # For "torus", usually means input domain is periodic.
                 # Let's assume simple addition for now.
                 pass
            else:
                 # Smooth shift: just substract shift (or add)?
                 # f(x - shift) moves optimum to +shift.
                 shifted_vec.append(val - s)
                 
        # Compute Base
        val = self.base_fn(shifted_vec)
        
        # Delay
        if self.delay_ms > 0:
            time.sleep(self.delay_ms / 1000.0)
            
        # Log
        now = time.perf_counter()
        elapsed = (now - self.start_time) * 1000.0
        
        if val < self.best_so_far:
            self.best_so_far = val
        
        hit = val <= self.threshold
        if hit:
            self.hit_threshold = True
            
        self.evals.append({
            "val": val,
            "best": self.best_so_far,
            "time": elapsed,
            "hit": hit
        })
        
        return val

# --- Runners ---

def run_arqon(method, config_base, objective, budget, seed):
    if not HAS_ARQON:
        return "skipped", "arqon_missing"
    
    # 1. Prepare Config
    # Create bounds based on dim
    dim = len(objective.shift_vec)
    bounds = {}
    for i in range(dim):
        # default bounds?
        # sphere/rosen usually unlimited? or [-5,5]?
        # Let's assume [-5, 5] for simplicity unless specified
        bounds[f"x{i}"] = {"min": -5.0, "max": 5.0, "scale": "Linear"}
        
    config = {
        "bounds": bounds,
        "budget": budget,
        "seed": seed,
        "probe_ratio": 0.2 if method == "arqon_full" else 1.0 # probe_only
    }
    
    # Init Solver/Probe
    if method == "arqon_probe_only":
        probe = arqonhpo.ArqonProbe(json.dumps(config), seed=seed)
        # Generate all points (batch)
        points = probe.sample_range(0, budget)
        for p in points:
            objective(p)
    else:
        solver = arqonhpo.ArqonSolver(json.dumps(config))
        for _ in range(budget):
            cands = solver.ask()
            if not cands: break
            results = []
            for c in cands:
                y = objective(c)
                # Matches Rust EvalTrace: eval_id, params, value, cost
                # Using deterministic or simple ID
                # We don't have global ID easily unless we track it
                # Using hash or timestamp? Or simple localized counter?
                # Benchmark doesn't strictly rely on ID uniqueness across runs, just validity.
                # Let's use simple random ID or hash of params
                eid = int(hashlib.md5(json.dumps(c, sort_keys=True).encode()).hexdigest()[:8], 16)
                results.append({
                    "eval_id": eid,
                    "params": c,
                    "value": y,
                    "cost": y
                })
            solver.tell(json.dumps(results))
            
    return "ok", None

def run_optuna(method, config_base, objective, budget, seed):
    if not HAS_OPTUNA:
        return "skipped", "optuna_missing"
    
    sampler = optuna.samplers.TPESampler(seed=seed)
    study = optuna.create_study(sampler=sampler, direction="minimize")
    
    dim = len(objective.shift_vec)
    
    def optuna_obj(trial):
        # Define params
        x = []
        for i in range(dim):
            x.append(trial.suggest_float(f"x{i}", -5.0, 5.0))
        return objective(x)
    
    study.optimize(optuna_obj, n_trials=budget)
    return "ok", None

def run_random(method, config_base, objective, budget, seed):
    rng = random.Random(seed)
    dim = len(objective.shift_vec)
    
    for _ in range(budget):
        # Uniform random
        x = [rng.uniform(-5.0, 5.0) for _ in range(dim)]
        # format as dict for consistency? objective handles list
        objective(x)
    return "ok", None


# --- Main Loop ---

def run_suite(suite_path, output_dir):
    with open(suite_path) as f:
        suite = json.load(f)
        
    suite_name = suite["suite_name"]
    base_seed = suite["seed"]
    dims = suite["dims"]
    n_shifts = suite.get("n_shifts", 1)
    base_budget = suite["budget"]
    horizon = suite["horizon"]
    
    # Initialize Logs
    os.makedirs(output_dir, exist_ok=True)
    runs_file = os.path.join(output_dir, "runs.csv")
    trace_file = os.path.join(output_dir, "trace.csv")
    summary_file = os.path.join(output_dir, "summary.csv")
    
    # Write Headers
    if not os.path.exists(runs_file):
        with open(runs_file, 'w') as f:
            csv.writer(f).writerow(RUNS_COLUMNS)
    if not os.path.exists(trace_file):
        with open(trace_file, 'w') as f:
            csv.writer(f).writerow(TRACE_COLUMNS)
            
    # Iterate
    profiles = suite["profiles"]
    workloads = suite["workloads"]
    methods = suite["methods"]
    
    total_tasks = len(profiles) * len(workloads) * len(methods) * n_shifts
    print(f"Starting Suite '{suite_name}': {total_tasks} runs.")
    
    for profile in profiles:
        prof_name = profile["name"]
        delay = profile["eval_delay_ms"]
        
        for workload in workloads:
            wl_name = workload["name"]
            threshold = workload["threshold"]
            policy = workload["shift_policy"]
            wl_fn = WORKLOADS.get(wl_name, sphere) # Default fallback
            
            for method in methods:
                 for shift_id in range(n_shifts):
                     # Deterministic Params
                     seed = base_seed + shift_id * 1000 # Simple offset? Or hash? 
                     # Better: hash seed+shift_id -> new int? 
                     # Let's keep offset for reproducibility simplicity.
                     
                     shift_vec = get_shift_vec(dims, base_seed, shift_id, policy)
                     
                     # Check if run exists? (Optimization for rerunning)
                     # Generate RunID
                     run_params = {
                         "suite": suite_name, "seed": seed, "method": method,
                         "workload": wl_name, "profile": prof_name, "shift": shift_id,
                         "params": {"dims": dims, "budget": base_budget}
                     }
                     run_id = deterministic_run_id(run_params)
                     
                     # Execute
                     print(f"Running {run_id} ({method} / {wl_name} / {prof_name})...")
                     
                     objective = ObjectiveWrapper(wl_name, wl_fn, shift_vec, policy, delay, threshold)
                     
                     start_ts = time.time() # UTC approx
                     status = "ok"
                     error = None
                     
                     try:
                         if method.startswith("arqon"):
                             status, error = run_arqon(method, suite, objective, base_budget, seed)
                         elif method == "optuna_tpe":
                             status, error = run_optuna(method, suite, objective, base_budget, seed)
                         elif method == "random":
                             status, error = run_random(method, suite, objective, base_budget, seed)
                         else:
                             status = "skipped"
                             error = "unknown_method"
                     except Exception as e:
                         status = "error"
                         error = str(e)
                         print(f"Error in {run_id}: {e}")

                     # Log Run
                     with open(runs_file, 'a') as f:
                         writer = csv.DictWriter(f, fieldnames=RUNS_COLUMNS)
                         writer.writerow({
                             COL_RUN_ID: run_id,
                             COL_SUITE_NAME: suite_name,
                             COL_TIMESTAMP_UTC: start_ts,
                             COL_SEED: seed,
                             COL_PROFILE_NAME: prof_name,
                             COL_EVAL_DELAY_MS: delay,
                             COL_METHOD: method,
                             COL_WORKLOAD: wl_name,
                             COL_DIMS: dims,
                             COL_BUDGET: base_budget,
                             COL_HORIZON: horizon,
                             COL_THRESHOLD: threshold,
                             COL_SHIFT_POLICY: policy,
                             COL_SHIFT_ID: shift_id,
                             COL_SHIFT_VEC: json.dumps(shift_vec),
                             COL_STATUS: status,
                             COL_ERROR: error or ""
                         })
                         
                     # Log Trace (if ok)
                     if status == "ok":
                         with open(trace_file, 'a') as f:
                             writer = csv.DictWriter(f, fieldnames=TRACE_COLUMNS)
                             # Batch write
                             rows = []
                             for i, ev in enumerate(objective.evals):
                                 hit_idx = 1 if ev["hit"] else 0
                                 hit_hor = 1 if ev["hit"] and (i+1) <= horizon else 0
                                 rows.append({
                                     COL_RUN_ID: run_id,
                                     COL_EVAL_IDX: i+1,
                                     COL_BEST_SO_FAR: ev["best"],
                                     COL_LAST_VAL: ev["val"],
                                     COL_TIME_MS_ELAPSED: f"{ev['time']:.3f}",
                                     COL_HIT_THRESHOLD: hit_idx,
                                     COL_HIT_BY_HORIZON: hit_hor
                                 })
                             writer.writerows(rows)

    # Post-processing
    print("Aggregating results...")
    agg.aggregate_results(runs_file, trace_file, summary_file)
    
    print("Generating plots...")
    plotter.generate_plots_with_runs(runs_file, trace_file, os.path.join(output_dir, "plots"))
    
    print("Done.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--suite", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    
    run_suite(args.suite, args.out)
