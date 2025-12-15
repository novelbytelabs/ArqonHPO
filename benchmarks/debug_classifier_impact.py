import json
import numpy as np
import arqonhpo
import sys

def sphere_smooth_shift(x, u_opt):
    real_x = ((x - u_opt + 0.5) * 2 - 1) * 5.0
    return float(np.sum(real_x**2))

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
            
    hit = evals_to_hit is not None and evals_to_hit <= horizon
    return hit

dims, budget = 5, 200
base_seed = 42
np.random.seed(base_seed)
seeds = [int(x) for x in np.random.randint(0, 100000, 30)]

print(f"Running Sphere Benchmark (30 seeds)...")
hits = 0
for seed in seeds:
    np.random.seed(seed)
    u_opt = np.random.rand(dims) * 0.4 + 0.3
    def fn(x): return sphere_smooth_shift(x, u_opt)
    if run_test(fn, dims, budget, 2.5, seed=seed):
        hits += 1

print(f"Sphere Hits: {hits}/30")
