# Inference Serving

ArqonHPO optimizes **LLM serving infrastructure** in real-time, adapting to traffic patterns and model behavior.

---

## The Challenge

LLM inference costs dominate AI budgets. Static configurations leave performance on the table:

- **Batch size too small** = GPU underutilized
- **Batch size too large** = p99 latency blows SLA
- **KV-cache miss** = expensive recomputation

ArqonHPO continuously tunes these parameters as traffic shifts.

---

## What to Optimize

| Parameter                | Impact                            |
| ------------------------ | --------------------------------- |
| `batch_size`             | Throughput vs. latency            |
| `max_tokens`             | Memory vs. completion length      |
| `kv_cache_size`          | Hit rate vs. memory               |
| `speculative_decoding_k` | Speed vs. accuracy                |
| `router_weights`         | Model selection (small vs. large) |

---

## Example: Dynamic Batch Sizing

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,
    "budget": 10000,
    "bounds": {
        "batch_size": {"min": 1, "max": 64, "scale": "Log"},
        "max_wait_ms": {"min": 10, "max": 500}
    }
}

solver = ArqonSolver(json.dumps(config))

while True:
    candidate = solver.ask_one()
    if candidate is None:
        break

    # Apply new batch config
    serving_engine.set_batch_config(
        batch_size=int(candidate["batch_size"]),
        max_wait_ms=candidate["max_wait_ms"]
    )

    # Measure over 60 seconds
    metrics = serving_engine.collect_metrics(duration_s=60)

    # Objective: maximize throughput while keeping p99 < SLA
    if metrics.p99_latency_ms > SLA_MS:
        reward = -1000  # Penalty for SLA violation
    else:
        reward = -metrics.cost_per_token  # Minimize cost

    solver.seed(json.dumps([{
        "params": candidate,
        "value": reward,
        "cost": 60.0  # Cost = seconds spent
    }]))
```

---

## Integration Points

### vLLM

```python
# Tune PagedAttention parameters
config["bounds"]["block_size"] = {"min": 8, "max": 32}
config["bounds"]["gpu_memory_utilization"] = {"min": 0.7, "max": 0.95}
```

### TensorRT-LLM

```python
# Tune inflight batching
config["bounds"]["max_batch_size"] = {"min": 1, "max": 256}
config["bounds"]["max_queue_delay_ms"] = {"min": 1, "max": 100}
```

### Triton Inference Server

```python
# Tune dynamic batching
config["bounds"]["preferred_batch_size"] = {"min": 1, "max": 128}
config["bounds"]["max_queue_delay_microseconds"] = {"min": 100, "max": 10000}
```

---

## Safety for Production

Use Guardrails to prevent catastrophic configurations:

```json
{
  "bounds": [
    [1, 64],
    [10, 500]
  ],
  "max_delta": [8, 100],
  "max_updates_per_second": 1.0
}
```

- **max_delta** prevents wild swings in batch size
- **rate limit** prevents oscillation

---

## Next Steps

- [Batch vs Online Mode](../documentation/concepts/batch_vs_online.md)
- [Safety Deep Dive](../documentation/concepts/safety.md)
- [Observability](../documentation/reference/observability.md)
