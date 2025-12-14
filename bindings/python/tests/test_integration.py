import json
import pytest
import math
from arqonhpo import ArqonSolver

def test_solver_basics():
    """Verify basic ask/tell loop and determinism."""
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {
            "x": {"min": -5.0, "max": 5.0, "scale": "Linear"}
        },
        "probe_ratio": 0.2
    }
    
    solver = ArqonSolver(json.dumps(config))
    
    # Phase 1: Probe
    candidates = solver.ask()
    assert candidates is not None
    assert len(candidates) > 0 # Should start with candidates
    
    # Check bounds
    c0 = candidates[0]
    assert -5.0 <= c0["x"] <= 5.0
    
    # Report back dummy results
    results = []
    for i, c in enumerate(candidates):
        # f(x) = x^2
        val = c["x"]**2
        results.append({
            "eval_id": i,
            "params": c,
            "value": val,
            "cost": 1.0
        })
    
    solver.tell(json.dumps(results))
    
    # Next Ask should trigger transition or more points
    # Since budget > probe count, should move to Classify -> Refine
    # Note: Refine strategy not implemented yet (Phase 2), so expect None or Done?
    # Logic: Classify -> Refine -> Strategy::step.
    # Current code returns None if strategy not set.
    
    next_step = solver.ask()
    # Expect None because strategy is missing in Phase 2
    assert next_step is None

    # Determinism check (Run 2)
    solver2 = ArqonSolver(json.dumps(config))
    candidates2 = solver2.ask()
    assert candidates == candidates2
