# Quickstart

## **From zero to optimized in 5 minutes.**

ArqonHPO is a high-performance hyperparameter optimization library that adapts to your problem automatically.

---

## Installation

```bash
pip install arqonhpo
```

Verify installation:

```bash
python -c "from arqonhpo import ArqonSolver; print('✓ Ready')"
```

Need other options? See the full [Installation Guide](installation.md).

---

## Core Concepts

Before diving in, here's what you need to know:

| Concept                | Description                                                          |
| ---------------------- | -------------------------------------------------------------------- |
| **Objective Function** | The function you want to minimize (e.g., loss, error, cost)          |
| **Parameters**         | The variables you want to optimize (e.g., learning_rate, batch_size) |
| **Bounds**             | The min/max range for each parameter                                 |
| **Budget**             | Total number of evaluations allowed                                  |
| **PCR Algorithm**      | ArqonHPO's adaptive algorithm: **P**robe → **C**lassify → **R**efine |

ArqonHPO automatically detects whether your problem is smooth (uses Nelder-Mead) or noisy (uses TPE), so you don't have to choose.

---

## Your First Optimization

Let's optimize a simple 2D function. The minimum is at `(2, -1)` — let's see if ArqonHPO finds it.

### Step 1: Define Your Objective Function

The **objective function** is what you want to minimize. It takes a dictionary of parameters and returns a single number (the "loss" or "cost").

```python
def objective(params: dict) -> float:
    """Simple quadratic bowl with minimum at (2, -1)."""
    x = params["x"]
    y = params["y"]
    return (x - 2)**2 + (y + 1)**2
```

!!! tip "Minimize, Not Maximize"
ArqonHPO always **minimizes**. To maximize something (like accuracy), return its negative: `return -accuracy`.

### Step 2: Configure the Solver

Create a configuration dictionary specifying your search space:

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,                    # Reproducibility
    "budget": 50,                  # Total evaluations
    "bounds": {
        "x": {"min": -10.0, "max": 10.0},
        "y": {"min": -10.0, "max": 10.0}
    }
}

solver = ArqonSolver(json.dumps(config))
```

### Step 3: Run the Optimization Loop

The ArqonHPO API uses an **ask-tell** interface:

1. **`ask()`** — Get a batch of parameter configurations to evaluate
2. **Evaluate** — Run your objective function on each configuration
3. **`tell()`** — Report the results back to the solver

```python
best_value = float('inf')
best_params = None

while True:
    # 1. Ask for candidates
    batch = solver.ask()
    if batch is None:
        break  # Budget exhausted

    # 2. Evaluate each candidate
    results = []
    for i, params in enumerate(batch):
        value = objective(params)

        # Track the best
        if value < best_value:
            best_value = value
            best_params = params

        # Record this evaluation
        results.append({
            "eval_id": i,
            "params": params,
            "value": value,
            "cost": 1.0  # Relative cost of this evaluation
        })

    # 3. Tell the solver what happened
    solver.tell(json.dumps(results))

print(f"✓ Best parameters: {best_params}")
print(f"✓ Best value: {best_value:.6f}")
```

**Expected output:**

```
[Machine] Classified as Structured (Score: 1.0172)
[Machine] Structured Fail-Safe Triggered! Restarting with CP Shift at param count 36
✓ Best parameters: {'x': 1.1, 'y': -0.6066}
✓ Best value: 0.964791
```

!!! note "Results May Vary"
The exact values depend on the solver's internal state. With only 50 evaluations, the solver is still exploring. Increase `budget` to 200+ for convergence closer to the true minimum at (2, -1).

🎉 **ArqonHPO is working!** The solver classified the landscape as "Structured" and is refining toward the optimum.

---

## Complete Example

Here's everything together as a copy-paste script:

```python
"""ArqonHPO Quickstart - Complete Example"""
import json
from arqonhpo import ArqonSolver

# 1. Define objective function (minimize this)
def objective(params: dict) -> float:
    x, y = params["x"], params["y"]
    return (x - 2)**2 + (y + 1)**2  # Minimum at (2, -1)

# 2. Configure solver
config = {
    "seed": 42,
    "budget": 50,
    "bounds": {
        "x": {"min": -10.0, "max": 10.0},
        "y": {"min": -10.0, "max": 10.0}
    }
}

solver = ArqonSolver(json.dumps(config))
best = {"value": float('inf'), "params": None}

# 3. Optimization loop
while True:
    batch = solver.ask()
    if batch is None:
        break

    results = []
    for i, params in enumerate(batch):
        value = objective(params)
        if value < best["value"]:
            best = {"value": value, "params": params}
        results.append({
            "eval_id": i,
            "params": params,
            "value": value,
            "cost": 1.0
        })

    solver.tell(json.dumps(results))

# 4. Results
print(f"Evaluations used: {solver.get_history_len()}")
print(f"Best x: {best['params']['x']:.4f}")
print(f"Best y: {best['params']['y']:.4f}")
print(f"Best value: {best['value']:.6f}")
```

**Expected output:**

```
[Machine] Classified as Structured (Score: 1.0172)
[Machine] Structured Fail-Safe Triggered! Restarting with CP Shift at param count 36
Evaluations used: 50
Best x: 1.1000
Best y: -0.6066
Best value: 0.964791
```

---

## Understanding the Configuration

### Required Fields

```python
config = {
    "seed": 42,       # Random seed for reproducibility
    "budget": 100,    # Maximum number of evaluations
    "bounds": {       # Parameter search space
        "param_name": {"min": 0.0, "max": 1.0}
    }
}
```

### Optional Fields

```python
config = {
    # ... required fields ...

    "probe_ratio": 0.2,  # Fraction of budget for initial exploration (default: 0.2)

    "strategy_params": {  # Fine-tune strategy behavior
        "gamma": 0.25,    # TPE quantile threshold
        "n_startup": 10   # Random samples before model-based optimization
    }
}
```

### Parameter Scales

ArqonHPO supports different parameter scales for different use cases:

=== "Linear (Default)"
`python
    "learning_rate": {"min": 0.001, "max": 0.1}
    `
Best for: Most parameters with uniform importance across the range.

=== "Log Scale"
`python
    "learning_rate": {"min": 0.0001, "max": 1.0, "scale": "Log"}
    `
Best for: Parameters spanning multiple orders of magnitude (learning rates, regularization).

=== "Periodic"
`python
    "angle": {"min": 0.0, "max": 360.0, "scale": "Periodic"}
    `
Best for: Angles, phases, or any parameter that wraps around.

---

## Understanding the Results

### Result Object Structure

Each result you `tell()` the solver should have:

```python
{
    "eval_id": 0,           # Identifier within this batch
    "params": {             # The parameters that were evaluated
        "x": 1.5,
        "y": -0.8
    },
    "value": 0.89,          # The objective function value (minimize this)
    "cost": 1.0             # Relative cost (for budget tracking)
}
```

### Cost Field

The `cost` field tells ArqonHPO how "expensive" each evaluation was to run. Think of it as a unit of work or time spent.

**Why does this matter?** ArqonHPO tracks your remaining budget by subtracting costs. When total cost reaches your `budget`, optimization stops. This lets you optimize for _wall-clock time_ rather than just counting evaluations.

- If all your evaluations take roughly the same time → set `cost: 1.0` for everything
- If some configurations are faster or slower → scale the cost proportionally

**Example scenario:** You're tuning a neural network. Small networks train in 10 seconds, large networks take 100 seconds.

```python
# Small network (fast) - costs less of your budget
results.append({"params": params, "value": loss, "cost": 0.1})

# Large network (slow) - costs more of your budget
results.append({"params": params, "value": loss, "cost": 1.0})
```

This way, ArqonHPO understands that 10 small-network evaluations ≈ 1 large-network evaluation in terms of real time spent.

**Quick reference:**

| Situation                               | What to use         |
| --------------------------------------- | ------------------- |
| All evaluations take the same time      | `cost: 1.0` for all |
| Evaluation took half the normal time    | `cost: 0.5`         |
| Evaluation took twice as long           | `cost: 2.0`         |
| You want to count evaluations, not time | `cost: 1.0` for all |

!!! tip "When in Doubt"
If you're not sure, just use `cost: 1.0` for everything. This makes the budget behave like a simple evaluation counter, which is fine for most use cases.

---

## Online (Real-time) Optimization

For real-time control systems where you need single-point feedback:

```python
from arqonhpo import ArqonSolver
import json

config = {
    "seed": 42,
    "budget": 1000,
    "bounds": {"gain": {"min": 0.1, "max": 10.0}}
}

solver = ArqonSolver(json.dumps(config))

for step in range(100):
    # Get ONE candidate at a time
    candidate = solver.ask_one()
    if candidate is None:
        break

    # Apply to real system
    reward = apply_to_system(candidate)

    # Immediate feedback via seed()
    solver.seed(json.dumps([{
        "params": candidate,
        "value": -reward,  # Negate reward for minimization
        "cost": 1.0
    }]))
```

!!! info "`ask_one()` vs `ask()`" - **`ask()`** returns a batch of candidates for the PCR workflow - **`ask_one()`** returns exactly one candidate for incremental/online optimization

---

## Direct Sampling with ArqonProbe

For advanced use cases like distributed computing or custom algorithms, use `ArqonProbe` for deterministic sampling:

```python
from arqonhpo import ArqonProbe
import json

config = {
    "seed": 42,
    "budget": 100,
    "bounds": {
        "x": {"min": -5.0, "max": 5.0},
        "y": {"min": -5.0, "max": 5.0}
    }
}

probe = ArqonProbe(json.dumps(config), seed=42)

# Sample at specific indices (deterministic, stateless)
sample_0 = probe.sample_at(0)
sample_1 = probe.sample_at(1)
print(f"Sample 0: {sample_0}")
print(f"Sample 1: {sample_1}")

# Sample a range (for parallel evaluation)
samples = probe.sample_range(0, 10)  # Indices 0-9
print(f"Got {len(samples)} samples")
```

**Why use ArqonProbe?**

| Use Case                  | Benefit                                        |
| ------------------------- | ---------------------------------------------- |
| **Distributed computing** | Each worker samples a specific index range     |
| **Reproducibility**       | Same index always produces same sample         |
| **Zero coordination**     | Workers don't need to communicate              |
| **Fault tolerance**       | Crashed workers can be restarted at same index |

---

## Warm-Starting from Previous Runs

Resume optimization from a previous run using `seed()`:

```python
import json
from arqonhpo import ArqonSolver

# Load previous results
previous_results = [
    {"params": {"x": 1.5, "y": -0.5}, "value": 0.5, "cost": 1.0},
    {"params": {"x": 2.1, "y": -1.2}, "value": 0.05, "cost": 1.0},
    {"params": {"x": 1.9, "y": -0.9}, "value": 0.02, "cost": 1.0},
]

config = {
    "seed": 42,
    "budget": 100,
    "bounds": {"x": {"min": -10.0, "max": 10.0}, "y": {"min": -10.0, "max": 10.0}}
}

solver = ArqonSolver(json.dumps(config))

# Seed with historical data
solver.seed(json.dumps(previous_results))

print(f"Started with {solver.get_history_len()} historical evaluations")

# Continue optimizing...
batch = solver.ask()
```

---

## Common Patterns

### Pattern 1: ML Hyperparameter Tuning

```python
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import cross_val_score
from arqonhpo import ArqonSolver
import json

X, y = load_your_data()

def objective(params):
    clf = RandomForestClassifier(
        n_estimators=int(params["n_estimators"]),
        max_depth=int(params["max_depth"]),
        min_samples_split=int(params["min_samples_split"]),
        random_state=42
    )
    score = cross_val_score(clf, X, y, cv=5).mean()
    return -score  # Minimize negative accuracy

config = {
    "seed": 42,
    "budget": 50,
    "probe_ratio": 0.3,  # More exploration for noisy ML objectives
    "bounds": {
        "n_estimators": {"min": 10, "max": 300},
        "max_depth": {"min": 2, "max": 30},
        "min_samples_split": {"min": 2, "max": 20}
    }
}

# ... standard optimization loop ...
```

### Pattern 2: Simulation Tuning

```python
def run_simulation(params):
    """Run expensive CFD/physics simulation."""
    result = simulator.run(
        velocity=params["velocity"],
        angle=params["angle"],
        pressure=params["pressure"]
    )
    return result.error  # Minimize error

config = {
    "seed": 42,
    "budget": 100,
    "bounds": {
        "velocity": {"min": 0.1, "max": 10.0},
        "angle": {"min": 0.0, "max": 90.0},
        "pressure": {"min": 100.0, "max": 1000.0}
    }
}
```

### Pattern 3: Neural Network Architecture Search

```python
def objective(params):
    model = build_model(
        hidden_size=int(params["hidden_size"]),
        num_layers=int(params["num_layers"]),
        dropout=params["dropout"],
        learning_rate=params["learning_rate"]
    )
    val_loss = train_and_evaluate(model)
    return val_loss

config = {
    "seed": 42,
    "budget": 30,  # Lower budget for expensive training
    "bounds": {
        "hidden_size": {"min": 32, "max": 512},
        "num_layers": {"min": 1, "max": 8},
        "dropout": {"min": 0.0, "max": 0.5},
        "learning_rate": {"min": 1e-5, "max": 1e-2, "scale": "Log"}
    }
}
```

---

## Debugging Tips

### Check Budget Usage

```python
print(f"Evaluations used: {solver.get_history_len()}")
```

### Verbose Objective Function

```python
def objective(params):
    print(f"Evaluating: x={params['x']:.4f}, y={params['y']:.4f}")
    value = (params["x"] - 2)**2 + (params["y"] + 1)**2
    print(f"  → value={value:.6f}")
    return value
```

### Common Mistakes

| Mistake                         | Symptom          | Fix                                       |
| ------------------------------- | ---------------- | ----------------------------------------- |
| Forgetting to call `tell()`     | Solver stalls    | Always call `tell()` after evaluating     |
| Returning `None` from objective | Crash            | Always return a number                    |
| Wrong JSON format               | ValueError       | Use `json.dumps()` for config and results |
| Bounds too tight                | Poor results     | Expand search space                       |
| Bounds too wide                 | Slow convergence | Narrow based on domain knowledge          |

---

## What's Next?

<div class="grid cards" markdown>

- :books: **[Python API Reference](reference/python.md)**

  Complete documentation of all classes and methods

- :book: **[Cookbook: ML Tuning](cookbook/ml_tuning.md)**

  Tune sklearn and PyTorch models

- :factory: **[Cookbook: Simulation Tuning](cookbook/sim_tuning.md)**

  Optimize expensive simulations

- :rocket: **[Cookbook: Real-time Control](cookbook/realtime.md)**

  Online optimization for control systems

- :brain: **[How PCR Works](concepts/pcr.md)**

  Deep dive into the Probe-Classify-Refine algorithm

- :wrench: **[CLI Usage](cli.md)**

  Use ArqonHPO from the command line

</div>

---

**Questions?** Check the [FAQ](faq.md) or [open an issue](https://github.com/novelbytelabs/ArqonHPO/issues).

**Found a bug?** [Report it](https://github.com/novelbytelabs/ArqonHPO/issues/new) and help us improve!

---
