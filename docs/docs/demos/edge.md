# Edge & Robotics

ArqonHPO is designed for **real-time control loops** where latency is measured in microseconds.

---

## The Challenge

Traditional HPO libraries have 100-1000ms overhead per trial. In a 1kHz control loop (1ms budget), this is impossible.

ArqonHPO overhead: **~3ms** for batch, **~100ns** per cached lookup.

---

## Use Cases

### PID Controller Tuning
Continuously adjust Kp, Ki, Kd gains based on tracking error.

```python
import json
from arqonhpo import ArqonSolver

config = {
    "seed": 42,
    "budget": 1000,
    "bounds": {
        "kp": {"min": 0.1, "max": 10.0},
        "ki": {"min": 0.0, "max": 1.0},
        "kd": {"min": 0.0, "max": 5.0}
    }
}

solver = ArqonSolver(json.dumps(config))

# In your control loop
while running:
    candidate = solver.ask_one()
    if candidate is None:
        break
    
    # Apply gains
    controller.set_gains(candidate["kp"], candidate["ki"], candidate["kd"])
    
    # Measure tracking error over N timesteps
    error = measure_tracking_error(duration_ms=100)
    
    # Feedback
    solver.seed(json.dumps([{
        "params": candidate,
        "value": error,  # Minimize error
        "cost": 1.0
    }]))
```

### Sensor Fusion Weights
Optimize weights for Kalman filter sensor fusion in real-time.

### Motor Control Parameters
Tune acceleration curves, jerk limits, and response damping.

---

## Embedded Deployment

ArqonHPO compiles to a static binary with no runtime dependencies:

```bash
# Cross-compile for ARM64
cargo build --release --target aarch64-unknown-linux-gnu -p arqonhpo-cli
```

Memory footprint: ~5MB binary, ~2MB runtime heap.

---

## Determinism

For safety-critical applications, ArqonHPO guarantees:

- **Reproducible sequences** with fixed seed
- **Bounded deltas** via Guardrails
- **Rollback** if performance regresses
- **Audit trail** of all parameter changes

---

## Next Steps

- [Safety Deep Dive](../documentation/concepts/safety.md) — Guardrails for safety-critical systems
