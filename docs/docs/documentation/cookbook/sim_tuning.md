# Cookbook: Simulation Tuning

Tune expensive, smooth simulation objectives with Nelder-Mead.

## Scenario

You have a CFD or physics simulation that:

- Takes **minutes to hours** per evaluation.
- Has a **smooth landscape** (small parameter changes = small output changes).
- You have a tight evaluation budget (e.g., 50-100 runs).

ArqonHPO will automatically detect this and use **Nelder-Mead**, which minimizes evaluations.

## The PCR Algorithm

For simulation tuning, ArqonHPO uses the **Probe-Classify-Refine (PCR)** algorithm:

1. **Probe**: It samples the parameter space using a multi-scale prime-index grid.
2. **Classify**: It analyzes the residuals of the probe phase. Smooth simulations show fast residual decay ($\alpha > 0.5$).
3. **Refine**: If structured, it launches **Nelder-Mead** initialized from the best probe points.

This "Hybrid Seeding" allows Nelder-Mead to start exploitation immediately from a high-quality region.
This means for CFD simulations with smooth, bowl-shaped objectives, ArqonHPO will:
- Detect geometric decay in residuals (fast convergence signature)
- Automatically select Nelder-Mead without manual configuration
- Use probe results to intelligently initialize the simplex

## Example: CFD Parameter Sweep

```python
import json
import time
from arqonhpo import ArqonSolver

# Simulate expensive CFD call
def cfd_simulation(params):
    inlet_velocity = params["inlet_velocity"]
    turbulence_k = params["turbulence_k"]
    
    time.sleep(0.5)  # Simulate 30-minute CFD; use 0.5s for demo
    
    # Fake "drag coefficient" as objective
    drag = (inlet_velocity - 5.0)**2 + (turbulence_k - 0.1)**2
    return drag

# Config
config = {
    "seed": 123,
    "budget": 30,  # Very tight budget
    "probe_ratio": 0.2,
    "bounds": {
        "inlet_velocity": {"min": 1.0, "max": 10.0},
        "turbulence_k": {"min": 0.01, "max": 0.5}
    }
}

solver = ArqonSolver(json.dumps(config))
best = {"value": float('inf')}

while True:
    batch = solver.ask()
    if batch is None:
        break
    
    results = []
    for i, params in enumerate(batch):
        drag = cfd_simulation(params)
        if drag < best["value"]:
            best = {"params": params, "value": drag}
        results.append({
            "eval_id": i,
            "params": params,
            "value": drag,
            "cost": 30.0  # 30 mins
        })
    
    solver.tell(json.dumps(results))

print(f"Optimal: {best}")
# Optimal: {'params': {'inlet_velocity': 5.02, 'turbulence_k': 0.098}, 'value': 0.0004}
```

## Why Nelder-Mead?

For smooth landscapes, Nelder-Mead's simplex operations converge faster than random search or TPE because it exploits local gradient information without needing derivatives.

### Nelder-Mead Operations

ArqonHPO implements all 5 standard NM operations:

| Operation | When Used | Formula |
|-----------|-----------|---------|
| **Reflection** | Always tried first | x_r = c + α(c - worst) |
| **Expansion** | Reflection is best so far | x_e = c + γ(r - c) |
| **Outside Contraction** | Reflection between 2nd-worst and worst | x_c = c + ρ(r - c) |
| **Inside Contraction** | Reflection worse than worst | x_c = c + ρ(worst - c) |
| **Shrink** | Contraction fails | x_i = best + σ(x_i - best) |

Standard coefficients: α=1.0, γ=2.0, ρ=0.5, σ=0.5

