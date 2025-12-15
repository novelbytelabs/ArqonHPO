import json
import numpy as np
import arqonhpo
import time

def sphere_smooth_shift(x, u_opt):
    real_x = ((x - u_opt + 0.5) * 2 - 1) * 5.0
    return float(np.sum(real_x**2))

def rosenbrock_smooth_shift(x, u_opt):
    real_x = ((x - u_opt + 0.5) * 2 - 1) * 2.048
    return float(np.sum(100 * (real_x[1:] - real_x[:-1]**2)**2 + (1 - real_x[:-1])**2))

def run_test(fn, dims, budget, threshold, horizon=100, seed=42):
    bounds = {f'x{i}': {'min': 0.0, 'max': 1.0, 'scale': 'Linear'} for i in range(dims)}
    config = {'seed': seed, 'budget': budget, 'bounds': bounds, 'probe_ratio': 0.2}
    solver = arqonhpo.ArqonSolver(json.dumps(config))
    best_val, counter = float('inf'), 0
    evals_to_hit = None
    
    while solver.get_history_len() < budget:
        candidates = solver.ask()
        if not candidates: break
        results = []
        for params in candidates:
            # Check budget
            if counter >= budget: break
            
            x = np.array([params[f'x{i}'] for i in range(dims)])
            val = fn(x)
            results.append({'eval_id': counter, 'params': params, 'value': val, 'cost': 0.0})
            counter += 1
            if val < best_val:
                best_val = val
                if evals_to_hit is None and best_val <= threshold:
                    evals_to_hit = counter
        
        if results:
            solver.tell(json.dumps(results))
            
    hit_by_100 = evals_to_hit is not None and evals_to_hit <= horizon
    return hit_by_100, evals_to_hit, best_val

def mean_median(lst):
    if not lst: return 0.0
    return np.median(lst)

dims, budget = 5, 200
base_seed = 42

print(f"Running A/B Benchmark (Dims={dims}, Budget={budget})")

# Sphere
print("\n--- Sphere ---")
sphere_hits = 0
sphere_evals = []
np.random.seed(base_seed)
seeds = [int(x) for x in np.random.randint(0, 100000, 30)]

for i, seed in enumerate(seeds):
    np.random.seed(seed)
    u_opt = np.random.rand(dims) * 0.4 + 0.3
    def fn(x): return sphere_smooth_shift(x, u_opt)
    
    hit, evals, best = run_test(fn, dims, budget, 2.5, seed=seed)
    if hit:
        sphere_hits += 1
        sphere_evals.append(evals)

print(f"Sphere Hits: {sphere_hits}/30")
print(f"Sphere Median Evals (hits only): {mean_median(sphere_evals):.1f}")

# Rosenbrock
print("\n--- Rosenbrock ---")
ros_hits = 0
ros_evals = []
np.random.seed(base_seed) # Reset to ensure same u_opt generation relative to seeds if we used that
# Actually, let's use the same seeds for reproducibility
for i, seed in enumerate(seeds):
    np.random.seed(seed)
    u_opt = np.random.rand(dims) * 0.4 + 0.3
    def fn(x): return rosenbrock_smooth_shift(x, u_opt)
    
    hit, evals, best = run_test(fn, dims, budget, 50.0, seed=seed)
    if hit:
        ros_hits += 1
        ros_evals.append(evals)

print(f"Rosenbrock Hits: {ros_hits}/30")
print(f"Rosenbrock Median Evals (hits only): {mean_median(ros_evals):.1f}")
