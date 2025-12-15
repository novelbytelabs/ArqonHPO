
import json
import time
import arqonhpo
import math

def run_experiment(name, is_periodic):
    print(f"\n--- Running Experiment: {name} (Periodic: {is_periodic}) ---")
    
    # Define function: smooth distance to 0.0 on circle OR linear line
    # If periodic, we treat [0, 1] as circle. target is 0.0.
    # dist(x, 0.0) = min(x, 1-x)
    # f(x) = dist(x, 0.0)^2
    # If NOT periodic (Linear), f(x) = x^2 (target 0.0)
    # But to show wrap effect, we need start at 0.9.
    # If linear optimizer starts at 0.9, and f(x) = (x - 0.0)^2, it will go 0.9 -> 0.8 ... -> 0.0.
    # That works for linear too.
    # We need a case where SHORT path wraps.
    # Target = 0.5. Start = 0.1 and 0.9.
    # Linear: 0.1 -> 0.5 (dist 0.4). 0.9 -> 0.5 (dist 0.4). Same.
    # Target = 0.05. Start = 0.9.
    # Linear: 0.9 -> ... -> 0.05 (dist 0.85).
    # Periodic: 0.9 -> 1.0/0.0 -> 0.05 (dist 0.15).
    # So Periodic should be FASTER (fewer evals) or SAME result?
    # Both converge to 0.05.
    # But Periodic path is shorter.
    # Better test: Function that has a BARRIER in linear space.
    # e.g. f(x) = 0.1 if x < 0.2, 1.0 if 0.2 < x < 0.8, 0.0 if x > 0.8.
    # Global limit at 0.9 (val 0.0). Local min at 0.1 (val 0.1).
    # Start at 0.1.
    # Linear: Stuck at 0.1.
    # Periodic: 0.1 -> 0.0 -> 0.9 -> 0.8 (downhill).
    # Function: "Circular Bowl with Barrier"
    # f(x) = cos(2*pi*x) ... minimum at ?
    # Let's use simple distance test first to prove `periodic_mask` is working.
    # If I pass `scale="Periodic"`, it should wrap.
    # If I pass `scale="Linear"`, it should clamp.
    # Test: x = 0.9. Step +0.2.
    # Linear -> 1.0.
    # Periodic -> 0.1.
    # How to verify?
    # I can spy on evaluations.
    # If I see 0.1, it wrapped.
    pass

def custom_objective(params):
    x = params['x']
    # Target is 0.1.
    # We encourage wrapping by starting at 0.9.
    # But we need to ensure the optimizer *tries* to wrap.
    # Simple Bowl at 0.1.
    # f(x) = min(|x - 0.1|, |(x - 0.1  - 1)|, |(x - 0.1 + 1)|)^2
    # dist on circle.
    
    diff = x - 0.1
    # wrap diff to [-0.5, 0.5]
    diff = diff - round(diff)
    dist = abs(diff)
    return dist * dist

# CONFIG A: Linear Domain
config_linear = {
    "bounds": {
        "x": {"min": 0.0, "max": 1.0, "scale": "Linear"}
    },
    "budget": 30,
    "seed": 42, # Seed 42 puts points?
    "probe_ratio": 0.1 # Minimal probe
}

# CONFIG B: Periodic Domain
config_periodic = {
    "bounds": {
        "x": {"min": 0.0, "max": 1.0, "scale": "Periodic"}
    },
    "budget": 30,
    "seed": 42,
    "probe_ratio": 0.1
}

def run_test(cfg_dict, label):
    print(f"\n[{label}] Running...")
    
    evaluated_points = []
    
    def internal_obj(params):
        x = params['x']
        evaluated_points.append(x)
        return custom_objective(params)

    # Serialize config
    cfg_json = json.dumps(cfg_dict)
    
    # Run
    # Note: python bindings might not expose `evaluate` callback easily if using `optimize`.
    # `arqonhpo.minimize`?
    # I need to check how to call it.
    # Assuming `arqonhpo.minimize(config_str, callback)`.
    # Based on previous interactions, it seems we use `Solver` class or similar?
    # Wait, existing python bindings in `lib.rs`: `state = State::new(config_json)`. `state.ask()`. `state.tell()`.
    # Let's check `lib.rs` interface if possible.
    # Assuming `arqonhpo.State`.
    
    try:
        # Initialize
        # Class is ArqonSolver
        # It is likely exposed directly or under _internal?
        # Usually: arqonhpo.ArqonSolver (if module export matches)
        # Check if ArqonSolver is available
        SolverClass = getattr(arqonhpo, "ArqonSolver", None)
        if SolverClass is None and hasattr(arqonhpo, "_internal"):
             SolverClass = getattr(arqonhpo._internal, "ArqonSolver", None)
             
        if SolverClass:
            solver = SolverClass(cfg_json)
            best_val = float('inf')
            eval_counter = 0
            
            while True:
                # Ask returns list of dicts (candidates) or None
                action = solver.ask()
                
                if action is None:
                    break
                
                candidates = action
                if not candidates:
                    break
                    
                full_results = []
                for cand in candidates:
                    val = internal_obj(cand)
                    eval_counter += 1
                    
                    # Construct EvalTrace dict
                    trace = {
                        "eval_id": eval_counter,
                        "params": cand,
                        "value": val,
                        "cost": 1.0
                    }
                    full_results.append(trace)
                    
                    if val < best_val:
                        best_val = val
                
                # Tell expects JSON string of list of EvalTrace
                solver.tell(json.dumps(full_results))
                
            print(f"[{label}] Best Value: {best_val:.6f}")
            mid_visits = sum(1 for x in evaluated_points if 0.4 < x < 0.6)
            print(f"[{label}] Points in [0.4, 0.6]: {mid_visits}")
            print(f"[{label}] Total points: {len(evaluated_points)}")
            
        else:
            print("Error: ArqonSolver class not found in arqonhpo module")
            print(f"Module dir: {dir(arqonhpo)}")
    except Exception as e:
        print(f"Error running {label}: {e}")

if __name__ == "__main__":
    run_test(config_linear, "Linear")
    run_test(config_periodic, "Periodic")
