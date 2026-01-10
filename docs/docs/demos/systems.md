# Systems & SRE

ArqonHPO keeps **infrastructure systems stable** by continuously tuning operational parameters as conditions change.

---

## The Challenge

Static configuration drifts from optimal as:
- Traffic patterns shift (day/night, seasonal)
- Hardware ages (slower disks, thermal throttling)
- Dependencies change (upstream latency, API limits)

Manual tuning can't keep up. ArqonHPO automates it.

---

## What to Optimize

### Database Connections
| Parameter | Impact |
|-----------|--------|
| `pool_size` | Throughput vs. connection overhead |
| `max_idle_time` | Resource usage vs. cold start |
| `connection_timeout` | Resilience vs. latency |

### Caching
| Parameter | Impact |
|-----------|--------|
| `ttl_seconds` | Freshness vs. hit rate |
| `max_memory_mb` | Hit rate vs. eviction |
| `eviction_policy` | Workload-specific optimization |

### Queue Management
| Parameter | Impact |
|-----------|--------|
| `batch_size` | Throughput vs. latency |
| `prefetch_count` | Throughput vs. memory |
| `visibility_timeout` | Reliability vs. reprocessing |

---

## Example: Connection Pool Tuning

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,
    "budget": 500,
    "bounds": {
        "pool_size": {"min": 5, "max": 100},
        "max_idle_time_s": {"min": 30, "max": 600},
        "connection_timeout_ms": {"min": 100, "max": 5000}
    }
}

solver = ArqonSolver(json.dumps(config))

while True:
    candidate = solver.ask_one()
    if candidate is None:
        break
    
    # Apply new pool config (hot reload)
    db_pool.reconfigure(
        pool_size=int(candidate["pool_size"]),
        max_idle_time=candidate["max_idle_time_s"],
        timeout=candidate["connection_timeout_ms"]
    )
    
    # Observe for 5 minutes
    metrics = observe_db_metrics(duration_s=300)
    
    # Multi-objective: high throughput, low p99, low errors
    score = (
        metrics.queries_per_second * 10
        - metrics.p99_latency_ms
        - metrics.error_rate * 1000
    )
    
    solver.seed(json.dumps([{
        "params": candidate,
        "value": -score,  # Minimize negative score = maximize
        "cost": 300.0
    }]))
```

---

## Example: JVM GC Tuning

```python
config = {
    "seed": 42,
    "budget": 100,
    "bounds": {
        "heap_size_gb": {"min": 4, "max": 32},
        "new_ratio": {"min": 1, "max": 8},
        "survivor_ratio": {"min": 2, "max": 32},
        "max_gc_pause_ms": {"min": 50, "max": 500}
    }
}

# Apply via JVM flags:
# -Xmx{heap_size}g -XX:NewRatio={new_ratio} ...
```

---

## Observability Integration

ArqonHPO exports Prometheus metrics:

```promql
# Track optimization progress
arqon_history_len
arqon_ask_total
arqon_tell_total

# Monitor for regressions
rate(arqon_eval_duration_seconds_sum[5m]) / rate(arqon_eval_duration_seconds_count[5m])
```

---

## Safety for Production

Guardrails prevent runaway configurations:

```json
{
  "max_delta": [10, 60, 500],
  "max_updates_per_second": 0.1,
  "rollback_on_regression": true
}
```

- **10 connections max change per update**
- **1 update per 10 seconds max**
- **Auto-rollback if p99 spikes**

---

## Next Steps

- [Safety Deep Dive](../documentation/concepts/safety.md)
- [Observability](../documentation/reference/observability.md)
- [CLI Reference](../documentation/reference/cli.md)
