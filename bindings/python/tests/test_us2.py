import json
import pytest
from arqonhpo import ArqonSolver
from fixtures.noisy import noisy_cheap, get_noisy_config

def test_us2_noisy_tuning_flow():
    """
    US2 Acceptance:
    - Probe -> Classify -> Chaotic Mode (TPE) -> Refine
    - Should handle noise better than random? Or just run.
    """
    config = get_noisy_config()
    solver = ArqonSolver(json.dumps(config))
    
    trace = []
    
    while len(trace) < config["budget"]:
        batch = solver.ask()
        if batch is None:
            break
        
        results = []
        for i, candidate in enumerate(batch):
            # Eval
            val = noisy_cheap(candidate)
            
            results.append({
                "eval_id": len(trace) + i,
                "params": candidate,
                "value": val,
                "cost": 0.1
            })
        
        solver.tell(json.dumps(results))
        trace.extend(results)
        
    # Validation checkpoints for US2
    assert len(trace) > 0
    
    prob_budget = int(config["budget"] * config["probe_ratio"])
    # Check if we ran past probe
    # This will FAIL until TPE (T023) is wired.
    assert len(trace) > prob_budget, "Solver stopped after probe phase (Strategy missing for Chaotic?)"
    
    # Optional: Verify mode was Chaotic? 
    # We can't query mode easily from Python yet without exposing it.
    # But the fact it ran implies it found a strategy.
    # If the classifier works as expected, it should have picked Chaotic.
