# Kubernetes Integration

Run ArqonHPO in Kubernetes for production hyperparameter optimization.

---

## Architecture Patterns

### Pattern 1: Sidecar

ArqonHPO runs as a sidecar container, optimizing your main application's parameters.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
spec:
  replicas: 1
  template:
    spec:
      containers:
        - name: app
          image: my-app:latest
          env:
            - name: CONFIG_PATH
              value: /shared/config.json
          volumeMounts:
            - name: shared
              mountPath: /shared
              
        - name: arqon
          image: arqonhpo:latest
          command:
            - arqonhpo
            - dashboard
            - --state=/shared/state.json
            - --addr=0.0.0.0:3030
          volumeMounts:
            - name: shared
              mountPath: /shared
          ports:
            - containerPort: 3030
              name: dashboard
            - containerPort: 9898
              name: metrics
              
      volumes:
        - name: shared
          emptyDir: {}
```

---

### Pattern 2: Job-based

Parallelize evaluations using Kubernetes Jobs.

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: arqon-eval-{{ .Candidate.ID }}
spec:
  template:
    spec:
      containers:
        - name: eval
          image: my-evaluator:latest
          env:
            - name: ARQON_x
              value: "{{ .Candidate.Params.x }}"
            - name: ARQON_y
              value: "{{ .Candidate.Params.y }}"
          command:
            - ./evaluate.sh
      restartPolicy: Never
```

**Controller Loop:**

```python
import json
import subprocess
from arqonhpo import ArqonSolver

solver = ArqonSolver(json.dumps(config))

while True:
    candidates = solver.ask()
    if not candidates:
        break
    
    # Launch K8s jobs for each candidate
    jobs = []
    for i, params in enumerate(candidates):
        job = launch_k8s_job(f"eval-{i}", params)
        jobs.append((job, params))
    
    # Wait and collect results
    results = []
    for job, params in jobs:
        value = wait_for_job_result(job)
        results.append({
            "params": params,
            "value": value,
            "cost": 1.0
        })
    
    solver.tell(json.dumps(results))
```

---

### Pattern 3: CronJob Tuning

Periodic re-optimization for drifting systems.

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: arqon-retune
spec:
  schedule: "0 */6 * * *"  # Every 6 hours
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: tune
              image: arqonhpo:latest
              command:
                - arqonhpo
                - run
                - --config=/config/config.json
                - --script=/scripts/evaluate.sh
                - --state=/state/state.json
              volumeMounts:
                - name: config
                  mountPath: /config
                - name: state
                  mountPath: /state
          restartPolicy: OnFailure
          volumes:
            - name: config
              configMap:
                name: arqon-config
            - name: state
              persistentVolumeClaim:
                claimName: arqon-state
```

---

## ConfigMap for Optimization Config

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: arqon-config
data:
  config.json: |
    {
      "seed": 42,
      "budget": 100,
      "bounds": {
        "batch_size": {"min": 1, "max": 64, "scale": "Log"},
        "learning_rate": {"min": 0.0001, "max": 0.1, "scale": "Log"}
      }
    }
```

---

## Prometheus Integration

ArqonHPO exposes metrics compatible with Prometheus Operator:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: arqon-metrics
spec:
  selector:
    matchLabels:
      app: arqon
  endpoints:
    - port: metrics
      interval: 15s
```

---

## PersistentVolumeClaim for State

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: arqon-state
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 100Mi
```

---

## Helm Chart (Planned)

We're working on an official Helm chart. Track progress: [Issue #XX](https://github.com/novelbytelabs/ArqonHPO/issues)

---

## Best Practices

1. **State persistence** — Use PVC for state files across restarts
2. **Resource limits** — ArqonHPO is lightweight (~50MB memory)
3. **Health checks** — Dashboard has `/api/summary` for liveness
4. **Secrets** — Use K8s Secrets for sensitive config values

---

## Next Steps

- [Observability](../reference/observability.md) — Prometheus metrics
- [Dashboard](../reference/dashboard.md) — REST API
- [Safety](../concepts/safety.md) — Guardrails for production
