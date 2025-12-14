# Cookbook: ML Model Tuning (US2)

Tune sklearn or PyTorch hyperparameters with TPE.

## Scenario

You have an ML training loop that:

- Is **cheap** (seconds per evaluation).
- Has a **noisy landscape** (variance from random seeds, data splits).
- You have a generous budget (e.g., 100-500 runs).

ArqonHPO will detect the noise/variance and use **TPE (Tree-structured Parzen Estimator)**.

## Example: Sklearn RandomForest

```python
import json
from sklearn.datasets import load_iris
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import cross_val_score
from arqonhpo import ArqonSolver

# Data
X, y = load_iris(return_X_y=True)

def objective(params):
    clf = RandomForestClassifier(
        n_estimators=int(params["n_estimators"]),
        max_depth=int(params["max_depth"]),
        random_state=42
    )
    # Cross-validation score (higher is better, so negate for minimization)
    score = cross_val_score(clf, X, y, cv=3).mean()
    return -score  # Minimize negative accuracy

# Config
config = {
    "seed": 42,
    "budget": 100,
    "probe_ratio": 0.3,  # More probing to detect noise
    "bounds": {
        "n_estimators": {"min": 10, "max": 200},
        "max_depth": {"min": 2, "max": 20}
    }
}

solver = ArqonSolver(json.dumps(config))
best = {"value": float('inf')}

while True:
    batch = solver.ask()
    if batch is None:
        break
    
    results = []
    for params in batch:
        loss = objective(params)
        if loss < best["value"]:
            best = {"params": params, "value": loss}
        results.append({
            "eval_id": 0,
            "params": params,
            "value": loss,
            "cost": 0.5
        })
    
    solver.tell(json.dumps(results))

print(f"Best: n_estimators={int(best['params']['n_estimators'])}, max_depth={int(best['params']['max_depth'])}")
print(f"Accuracy: {-best['value']:.4f}")
# Best: n_estimators=120, max_depth=8
# Accuracy: 0.9667
```

## Why TPE?

TPE builds probabilistic models of "good" and "bad" regions of the hyperparameter space, making it robust to noise and efficient at exploration.
