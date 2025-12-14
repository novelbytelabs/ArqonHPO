import json
import pytest
from arqonhpo import ArqonSolver
from fixtures.smooth import smooth_expensive, get_smooth_config

def test_us1_sim_tuning_flow():
    """
    US1 Acceptance:
    - Probe -> Classify -> Structured Mode (Nelder-Mead) -> Refine
    - Should converge towards 0.0 better than random.
    """
    config = get_smooth_config()
    solver = ArqonSolver(json.dumps(config))
    
    # Run loop
    best_value = float('inf')
    
    # We loop until Done (ask returns None)
    # Budget 50.
    
    # Note: In Phase 3 implementation steps T015-T016, we will wire Nelder-Mead.
    # Currently (start of Phase 3), the solver will return None after classify because Refine has no strategy.
    # This test asserts the *expected* behavior once implemented.
    # It will FAIL until T016 is done.
    
    trace = []
    
    while len(trace) < config["budget"]:
        batch = solver.ask()
        if batch is None:
            break
        
        results = []
        for i, candidate in enumerate(batch):
            # Eval
            val = smooth_expensive(candidate, sleep_time=0.0) # Fast for test
            
            best_value = min(best_value, val)
            results.append({
                "eval_id": len(trace) + i,
                "params": candidate,
                "value": val,
                "cost": 1.0 # placeholder
            })
        
        solver.tell(json.dumps(results))
        trace.extend(results)
        
    # Validation checkpoints for US1
    # 1. Did we run?
    assert len(trace) > 0
    # 2. Did we transition to Structured Mode? (Implicit via convergence check or future telemetry check)
    # T016 will wire Nelder-Mead.
    # For now, simplistic check: 
    # If using Nelder-Mead, we expect to find updates that are NOT just random samples.
    # But for a TDD test, "Running to completion" and "Better than pure probe" is a good start.
    
    prob_budget = int(config["budget"] * config["probe_ratio"])
    # Check if we ran past probe
    assert len(trace) > prob_budget, "Solver stopped after probe phase (Strategy missing?)"
    
    print(f"Best Value: {best_value}")
    # Threshold for Sphere[-5,5] with Budget 50 should be < 2.5 easily.
    assert best_value < 2.5
