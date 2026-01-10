# Determinism & Reproducibility

ArqonHPO guarantees **deterministic execution** — the same seed produces the same sequence of parameter suggestions.

---

## Why Determinism Matters

- **Debugging**: Reproduce exact failure conditions
- **Testing**: Verify optimizer behavior in CI
- **Auditing**: Explain why a specific config was chosen
- **Compliance**: Required in regulated industries

---

## RNG Seeding

ArqonHPO uses ChaCha8Rng (cryptographically secure, deterministic):

```python
from arqonhpo import ArqonSolver
import json

# Same seed = identical sequence
config1 = {"seed": 42, "budget": 100, "bounds": {...}}
config2 = {"seed": 42, "budget": 100, "bounds": {...}}

solver1 = ArqonSolver(json.dumps(config1))
solver2 = ArqonSolver(json.dumps(config2))

batch1 = solver1.ask()
batch2 = solver2.ask()

assert batch1 == batch2  # ✓ Always true
```

### Seed Requirements

| Type | Range | Notes |
|------|-------|-------|
| Python | `u64` (0 to 2^64-1) | Passed in config JSON |
| Rust | `u64` | Via `SolverConfig` |

---

## Reproducibility Conditions

For exact reproducibility, ensure:

1. **Same seed** — Obviously
2. **Same config** — All bounds, budget, params identical
3. **Same ArqonHPO version** — Algorithm changes break reproducibility
4. **Same tell() order** — Results must arrive in same sequence

### What Breaks Reproducibility

| Cause | Solution |
|-------|----------|
| Different ArqonHPO version | Pin version in requirements |
| Parallel evaluation with variable order | Sort results before `tell()` |
| Floating-point differences across platforms | Use identical hardware/OS |
| Different config order in JSON | Shouldn't matter, but use sorted keys |

---

## ArqonProbe: Stateless Sampling

`ArqonProbe` generates **deterministic, stateless** samples from Low-Discrepancy Sequences:

```python
from arqonhpo import ArqonProbe
import json

config = json.dumps({
    "bounds": {
        "x": {"min": 0, "max": 1},
        "y": {"min": 0, "max": 1}
    }
})

probe = ArqonProbe(config, seed=42)

# Stateless: any index returns same point
point_0 = probe.sample_at(0)      # Always same
point_1000 = probe.sample_at(1000)  # Always same

# Multiple calls at same index → same result
assert probe.sample_at(0) == probe.sample_at(0)  # ✓
```

### Zero-Coordination Sharding

Workers can generate disjoint ranges without communication:

```python
# Worker 0: indices 0-999
# Worker 1: indices 1000-1999
# Worker 2: indices 2000-2999

def worker(worker_id, probe, chunk_size=1000):
    start = worker_id * chunk_size
    return probe.sample_range(start, chunk_size)
```

Each worker produces deterministic, non-overlapping points.

---

## Internal Determinism

### Probe Phase
- LDS uses prime-sqrt-slopes rotation
- Sequence is purely deterministic from seed
- No randomness in Classify phase

### Refine Phase
- Nelder-Mead: fully deterministic simplex operations
- TPE: uses seeded RNG for kernel sampling
- SPSA: uses seeded RNG for perturbations

---

## Testing Determinism

Add to your test suite:

```python
def test_solver_determinism():
    config = json.dumps({
        "seed": 12345,
        "budget": 50,
        "bounds": {"x": {"min": -5, "max": 5}}
    })
    
    solver1 = ArqonSolver(config)
    solver2 = ArqonSolver(config)
    
    for _ in range(10):
        batch1 = solver1.ask()
        batch2 = solver2.ask()
        
        if batch1 is None and batch2 is None:
            break
            
        assert batch1 == batch2, "Determinism violated!"
        
        # Same fake results
        results = json.dumps([{
            "eval_id": i,
            "params": p,
            "value": sum(p.values()),
            "cost": 1.0
        } for i, p in enumerate(batch1)])
        
        solver1.tell(results)
        solver2.tell(results)
```

---

## Next Steps

- [Strategies](strategies.md) — How each algorithm maintains determinism
- [ArqonProbe](probe_deep_dive.md) — LDS implementation details
- [Safety](safety.md) — Audit trails for parameter changes
