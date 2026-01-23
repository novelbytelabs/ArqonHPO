# Ray Tune Integration

Use ArqonHPO as a backend for Ray Tune distributed hyperparameter optimization.

---

## Why ArqonHPO + Ray?

| Feature         | ArqonHPO Alone  | Ray Tune + ArqonHPO |
| --------------- | --------------- | ------------------- |
| Parallelism     | Manual          | Automatic           |
| Distributed     | Manual sharding | Built-in            |
| Fault tolerance | Manual          | Checkpointing       |
| Scheduling      | None            | ASHA, HyperBand     |

---

## Basic Integration

ArqonHPO can be used as a custom search algorithm in Ray Tune:

```python
from ray import tune
from ray.tune.search import Searcher
from arqonhpo import ArqonSolver
import json

class ArqonSearcher(Searcher):
    def __init__(self, config, **kwargs):
        super().__init__(**kwargs)
        self.solver = ArqonSolver(json.dumps(config))
        self._pending = {}

    def suggest(self, trial_id):
        candidate = self.solver.ask_one()
        if candidate is None:
            return Searcher.FINISHED
        self._pending[trial_id] = candidate
        return candidate

    def on_trial_complete(self, trial_id, result, **kwargs):
        params = self._pending.pop(trial_id)
        self.solver.seed(json.dumps([{
            "params": params,
            "value": result["loss"],
            "cost": 1.0
        }]))

# Usage
def train_fn(config):
    # Your training code
    loss = train_model(config["lr"], config["batch_size"])
    return {"loss": loss}

arqon_config = {
    "seed": 42,
    "budget": 100,
    "bounds": {
        "lr": {"min": 0.0001, "max": 0.1, "scale": "Log"},
        "batch_size": {"min": 8, "max": 128, "scale": "Log"}
    }
}

searcher = ArqonSearcher(arqon_config)

analysis = tune.run(
    train_fn,
    num_samples=100,
    search_alg=searcher,
    resources_per_trial={"cpu": 1, "gpu": 0.5}
)
```

---

## With Early Stopping (ASHA)

Combine ArqonHPO suggestions with ASHA scheduler:

```python
from ray.tune.schedulers import ASHAScheduler

scheduler = ASHAScheduler(
    max_t=100,
    grace_period=10,
    reduction_factor=2
)

analysis = tune.run(
    train_fn,
    num_samples=100,
    search_alg=ArqonSearcher(arqon_config),
    scheduler=scheduler,  # ASHA stops bad trials early
)
```

---

## Distributed ArqonProbe

For large parameter spaces, use `ArqonProbe` for embarrassingly parallel sampling:

```python
from arqonhpo import ArqonProbe
from ray import remote
import json

config = json.dumps({
    "bounds": {
        "lr": {"min": 0.0001, "max": 0.1},
        "bs": {"min": 8, "max": 128}
    }
})

@remote
def evaluate_sample(probe_config, seed, index):
    probe = ArqonProbe(probe_config, seed=seed)
    params = probe.sample_at(index)
    return {"params": params, "value": train_and_eval(params)}

# Launch 1000 parallel evaluations
futures = [
    evaluate_sample.remote(config, 42, i)
    for i in range(1000)
]
results = ray.get(futures)
```

Each worker generates deterministic, non-overlapping samples without coordination.

---

## Checkpointing

Save ArqonHPO state with Ray Tune checkpoints:

```python
class ArqonSearcherWithCheckpoint(ArqonSearcher):
    def save(self, checkpoint_path):
        # Export solver state
        state = self.solver.export()
        with open(checkpoint_path, "w") as f:
            f.write(state)

    def restore(self, checkpoint_path):
        with open(checkpoint_path) as f:
            state = f.read()
        self.solver = ArqonSolver.from_state(state)
```

---

## Resource Optimization

Use ArqonHPO to optimize Ray cluster resources:

```python
arqon_config = {
    "seed": 42,
    "budget": 50,
    "bounds": {
        "num_workers": {"min": 1, "max": 32},
        "cpus_per_worker": {"min": 1, "max": 8},
        "memory_per_worker_gb": {"min": 2, "max": 32}
    }
}
```

---

## Next Steps

- [Python API](../reference/python.md) — ArqonSolver/ArqonProbe
- [Determinism](../concepts/determinism.md) — Sharding pattern
- [Kubernetes](kubernetes.md) — Deploy Ray on K8s
