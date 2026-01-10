# Determinism & Reproducibility

**If you cannot replay history, you cannot debug the future.**

In stochastic optimization, "randomness" is often treated as a black box. But when an optimizer drives production infrastructure, "it worked differently last time" is an unacceptable root cause analysis.

ArqonHPO guarantees **bit-perfect deterministic execution**. Given the same seed and configuration, it will produce the exact same sequence of parameter suggestions, decision boundaries, and internal state transitions—forever, on any platform. This transforms optimization from a probabilistic gamble into a reproducible engineering process.

---

## Why Determinism Matters

Deterministic optimization is not just about convenience; it is a hard requirement for safety-critical autonomy:

| Use Case       | Requirement                                                        |
| -------------- | ------------------------------------------------------------------ |
| **Debugging**  | Reproduce exact failure conditions by replaying with the same seed |
| **Testing**    | Verify optimizer behavior in CI pipelines                          |
| **Auditing**   | Explain why a specific configuration was chosen                    |
| **Compliance** | Required in regulated industries (finance, medical, aerospace)     |
| **Research**   | Enable reproducible experiments and publications                   |

---

## RNG Implementation

ArqonHPO uses **ChaCha8Rng** from the `rand_chacha` crate (verified from `rng.rs`):

```rust
// From crates/core/src/rng.rs
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

pub fn get_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}
```

ChaCha8 is a cryptographically secure, portable pseudorandom number generator that produces identical sequences across platforms when given the same seed.

### Seed Configuration

```python
from arqonhpo import ArqonSolver
import json

# Same seed = identical sequence
config = {
    "seed": 42,        # u64 seed
    "budget": 100,
    "bounds": {"x": {"min": -5, "max": 5}}
}

solver = ArqonSolver(json.dumps(config))
```

| Field  | Type  | Range       |
| ------ | ----- | ----------- |
| `seed` | `u64` | 0 to 2^64-1 |

---

## Reproducibility Guarantees

Given identical inputs, ArqonHPO produces identical outputs:

```python
# Two solvers with same config
solver1 = ArqonSolver(json.dumps(config))
solver2 = ArqonSolver(json.dumps(config))

batch1 = solver1.ask()
batch2 = solver2.ask()

assert batch1 == batch2  # ✓ Always true
```

### What's Deterministic

| Component                | Deterministic | Notes                                  |
| ------------------------ | ------------- | -------------------------------------- |
| Probe phase              | ✅ Yes        | LDS uses prime-sqrt-slopes rotation    |
| Classify phase           | ✅ Yes        | Pure computation, no randomness        |
| Nelder-Mead              | ✅ Yes        | Fully deterministic simplex operations |
| TPE sampling             | ✅ Yes        | Uses seeded ChaCha8Rng                 |
| Cranley-Patterson shifts | ✅ Yes        | Derived from seed                      |

---

## Conditions for Reproducibility

For exact reproducibility, ensure:

1. **Same seed** in config
2. **Same config** (all bounds, budget, params identical)
3. **Same ArqonHPO version** (algorithm changes may affect sequences)
4. **Same `tell()` order** (results must arrive in same sequence)

### What Can Break Reproducibility

| Issue                     | Cause                                            | Solution                                |
| ------------------------- | ------------------------------------------------ | --------------------------------------- |
| Different version         | Algorithm updates                                | Pin version in requirements.txt         |
| Variable evaluation order | Parallel evaluation returning in different order | Sort results by eval_id before `tell()` |
| Platform differences      | Floating-point implementation                    | Use same platform (rare issue)          |
| Config ordering           | JSON key order                                   | Shouldn't matter, but use sorted keys   |

---

## ArqonProbe: Stateless Sampling

`ArqonProbe` generates **deterministic, stateless** samples. Unlike the solver, it doesn't maintain state—any index can be queried at any time with consistent results.

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

# Stateless: any index returns the same point
point_0 = probe.sample_at(0)        # Always same
point_1000 = probe.sample_at(1000)  # Always same

# Multiple calls at same index → identical result
assert probe.sample_at(0) == probe.sample_at(0)  # ✓
```

### Zero-Coordination Parallelism

The stateless property enables embarrassingly parallel sampling without worker communication:

```python
def worker(worker_id, probe, chunk_size=1000):
    """Each worker samples a disjoint range."""
    start = worker_id * chunk_size
    return probe.sample_range(start, chunk_size)

# Worker 0: indices 0-999
# Worker 1: indices 1000-1999
# Worker 2: indices 2000-2999
# No coordination needed!
```

Each worker produces deterministic, non-overlapping points. Results can be combined in any order without affecting reproducibility.

---

## Testing Determinism

Add this to your test suite to verify determinism:

```python
def test_solver_determinism():
    """Verify that two solvers with same seed produce identical sequences."""
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

        # Feed identical results
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

<div class="grid cards" markdown>

- :dart: **[Probe Deep Dive](probe_deep_dive.md)**

  LDS implementation and mathematics

- :gear: **[Strategies](strategies.md)**

  How each algorithm maintains determinism

- :shield: **[Safety](safety.md)**

  Audit trails for parameter changes

</div>

---
