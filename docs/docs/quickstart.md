# Quickstart

Get ArqonHPO running in 5 minutes.

## Installation

```bash
pip install arqonhpo
```

## Your First Optimization

### 1. Define your objective function

```python
def objective(params):
    x = params["x"]
    y = params["y"]
    return (x - 2)**2 + (y + 1)**2  # Minimum at (2, -1)
```

### 2. Configure the solver

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,
    "budget": 50,
    "bounds": {
        "x": {"min": -10.0, "max": 10.0},
        "y": {"min": -10.0, "max": 10.0}
    }
}

solver = ArqonSolver(json.dumps(config))
```

### 3. Run the optimization loop

```python
best_value = float('inf')
best_params = None

while True:
    batch = solver.ask()
    if batch is None:
        break
    
    results = []
    for params in batch:
        value = objective(params)
        if value < best_value:
            best_value = value
            best_params = params
        results.append({
            "eval_id": len(results),
            "params": params,
            "value": value,
            "cost": 1.0
        })
    
    solver.tell(json.dumps(results))

print(f"Best: {best_params} -> {best_value}")
# Best: {'x': 2.01, 'y': -0.98} -> 0.0005
```

## Next Steps

- [Cookbook: Sim Tuning](cookbook/sim_tuning.md) - Expensive CFD/Physics simulations
- [Cookbook: ML Tuning](cookbook/ml_tuning.md) - Sklearn/PyTorch hyperparameters
- [Python API Reference](reference/python.md)
