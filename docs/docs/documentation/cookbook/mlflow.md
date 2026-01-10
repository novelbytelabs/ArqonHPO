# MLflow Integration

Track ArqonHPO optimization runs with MLflow.

---

## Basic Setup

```python
import mlflow
from arqonhpo import ArqonSolver
import json

# Start MLflow run
mlflow.set_experiment("hyperparameter-tuning")

with mlflow.start_run(run_name="arqon-optimization"):
    config = {
        "seed": 42,
        "budget": 100,
        "bounds": {
            "lr": {"min": 0.0001, "max": 0.1, "scale": "Log"},
            "batch_size": {"min": 8, "max": 128}
        }
    }

    # Log config as params
    mlflow.log_params({
        "arqon_seed": config["seed"],
        "arqon_budget": config["budget"],
        "optimizer": "arqonhpo"
    })

    solver = ArqonSolver(json.dumps(config))

    iteration = 0
    while True:
        candidates = solver.ask()
        if candidates is None:
            break

        for params in candidates:
            # Evaluate
            value = train_model(params)

            # Log each evaluation
            mlflow.log_metrics({
                "objective": value,
                "lr": params["lr"],
                "batch_size": params["batch_size"]
            }, step=iteration)

            # Tell ArqonHPO
            solver.seed(json.dumps([{
                "params": params,
                "value": value,
                "cost": 1.0
            }]))

            iteration += 1

    # Log best result
    best = min(results, key=lambda x: x["value"])
    mlflow.log_metrics({"best_objective": best["value"]})
    mlflow.log_params({"best_" + k: v for k, v in best["params"].items()})
```

---

## Log Optimization Artifact

Export and log ArqonHPO state as an artifact:

```python
import tempfile
import os

# After optimization
with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
    artifact = solver.export()
    f.write(artifact)
    artifact_path = f.name

mlflow.log_artifact(artifact_path, "arqon")
os.unlink(artifact_path)
```

---

## Custom MLflow Callback

Create a reusable callback pattern:

```python
class MLflowCallback:
    def __init__(self, experiment_name, run_name=None):
        mlflow.set_experiment(experiment_name)
        self.run = mlflow.start_run(run_name=run_name)
        self.step = 0

    def on_ask(self, candidates):
        mlflow.log_metric("candidates_requested", len(candidates), step=self.step)

    def on_tell(self, results):
        for result in results:
            mlflow.log_metrics({
                "objective": result["value"],
                **{f"param_{k}": v for k, v in result["params"].items()}
            }, step=self.step)
            self.step += 1

    def on_complete(self, best):
        mlflow.log_metrics({"best_objective": best["value"]})
        mlflow.log_params({f"best_{k}": v for k, v in best["params"].items()})
        mlflow.end_run()

# Usage
callback = MLflowCallback("my-experiment", "run-001")

while True:
    candidates = solver.ask()
    if candidates is None:
        break

    callback.on_ask(candidates)
    results = evaluate_batch(candidates)
    solver.tell(json.dumps(results))
    callback.on_tell(results)

callback.on_complete(best)
```

---

## MLflow UI

View optimization runs:

```bash
mlflow ui --port 5000
```

Navigate to `http://localhost:5000` to see:

- Parameter progression
- Objective value over time
- Best configurations

---

## Compare Runs

```python
import mlflow

# Get experiment
experiment = mlflow.get_experiment_by_name("hyperparameter-tuning")

# Query runs
runs = mlflow.search_runs(
    experiment_ids=[experiment.experiment_id],
    order_by=["metrics.best_objective ASC"]
)

# Best run
best_run = runs.iloc[0]
print(f"Best run: {best_run['run_id']}")
print(f"Best objective: {best_run['metrics.best_objective']}")
```

---

## Docker Deployment

```dockerfile
FROM python:3.11-slim

RUN pip install arqonhpo mlflow

COPY train.py .

ENV MLFLOW_TRACKING_URI=http://mlflow-server:5000

CMD ["python", "train.py"]
```

---

## Next Steps

- [Ray Tune](ray.md) — Distributed optimization
- [Integrations](../integrations/index.md) — All integrations
- [Observability](../reference/observability.md) — Prometheus metrics
