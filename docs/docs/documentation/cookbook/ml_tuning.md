# Cookbook: ML Model Tuning

Tune sklearn or PyTorch hyperparameters with TPE.

## The PCR Algorithm for ML

While ArqonHPO is known for fast simulation tuning, it excels at ML tuning via the **PCR (Probe-Classify-Refine)** algorithm:

1. **Probe**: Scans the hyperparameter space.
2. **Classify**: **ResidualDecayClassifier** detects that ML loss surfaces are chaotic/noisy (slow residual decay, $\alpha \le 0.5$).
3. **Refine**: Automatically selects **TPE (Tree-structured Parzen Estimator)** instead of Nelder-Mead.

When probe samples show flat or irregular residual patterns (no geometric decay), ArqonHPO classifies the landscape as **Chaotic** and selects TPE:

- **α ≤ 0.5** → Many local optima, noisy evaluations → **TPE**
- TPE models "good" (l(x)) and "bad" (g(x)) distributions using kernel density estimation
- Samples are drawn to maximize Expected Improvement (EI)

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

## Scott's Rule Bandwidth

ArqonHPO uses **Scott's Rule** for adaptive kernel bandwidth in TPE:

```
σ = 1.06 × stddev × n^(-1/5)
```

This provides:

- **Automatic adaptation**: Bandwidth shrinks as more samples are collected
- **Data-driven scaling**: Uses sample standard deviation, not fixed percentages
- **Asymptotic optimality**: Minimizes mean integrated squared error for Gaussian kernels

Compared to fixed bandwidth (e.g., 10% of range):

| Method           | Pros                                                      | Cons                                     |
| ---------------- | --------------------------------------------------------- | ---------------------------------------- |
| **Scott's Rule** | Adapts to data distribution, optimal for smooth densities | May under-smooth in tails                |
| **Fixed 10%**    | Simple, predictable                                       | Ignores data structure, often suboptimal |

ArqonHPO defaults to Scott's Rule but supports alternatives via `BandwidthRule`:

```rust
TPE::with_bandwidth_rule(dim, BandwidthRule::Scott)    // Default
TPE::with_bandwidth_rule(dim, BandwidthRule::Silverman)  // Alternative
TPE::with_bandwidth_rule(dim, BandwidthRule::Fixed(0.1)) // Legacy behavior
```
