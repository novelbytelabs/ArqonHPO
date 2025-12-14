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
    
    # Next Ask should trigger transition to Refine phase
    # Since budget > probe count, should move to Classify -> Refine
    # Strategy IS now implemented (Nelder-Mead for smooth functions),
    # so it should return new candidates for evaluation.
    
    next_step = solver.ask()
    # Now that strategies are implemented, we expect candidates not None
    assert next_step is not None
    assert len(next_step) > 0

    # Determinism check (Run 2)
    solver2 = ArqonSolver(json.dumps(config))
    candidates2 = solver2.ask()
    assert candidates == candidates2
