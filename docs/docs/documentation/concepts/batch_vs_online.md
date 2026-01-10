# Batch vs Online Mode

ArqonHPO supports two primary modes of operation depending on your workflow.

---

## Mode Comparison

| Aspect | Batch Mode | Online Mode |
|--------|------------|-------------|
| **API** | `ask()` / `tell()` | `ask_one()` / `seed()` |
| **Candidates per call** | Multiple (batch) | Single |
| **Algorithm** | Full PCR | Direct TPE |
| **Best for** | Offline experiments | Real-time tuning |
| **Parallelism** | Evaluate batch in parallel | Sequential |

---

## Batch Mode (PCR Workflow)

The default mode uses **Probe → Classify → Refine**:

```python
from arqonhpo import ArqonSolver
import json

config = {
    "seed": 42,
    "budget": 100,
    "bounds": {"x": {"min": -5, "max": 5}}
}

solver = ArqonSolver(json.dumps(config))

while True:
    # Get a BATCH of candidates
    batch = solver.ask()
    if batch is None:
        break
    
    # Evaluate all candidates (can parallelize!)
    results = []
    for params in batch:
        value = objective(params)
        results.append({
            "eval_id": len(results),
            "params": params,
            "value": value,
            "cost": 1.0
        })
    
    solver.tell(json.dumps(results))
```

### Phases

1. **Probe (20% of budget)**: LDS sampling for landscape coverage
2. **Classify**: Analyze results, select strategy
3. **Refine (80% of budget)**: Execute chosen strategy

### When To Use

- ✅ Parallel evaluation possible
- ✅ Full budget known upfront
- ✅ Offline experimentation
- ✅ Want automatic strategy selection

---

## Online Mode (Real-time Workflow)

For **streaming** or **real-time** optimization, use `ask_one()`:

```python
solver = ArqonSolver(json.dumps(config))

while True:
    # Get ONE candidate
    candidate = solver.ask_one()
    if candidate is None:
        break
    
    # Evaluate immediately
    value = objective(candidate)
    
    # Feed back immediately
    solver.seed(json.dumps([{
        "params": candidate,
        "value": value,
        "cost": 1.0
    }]))
```

### Behavior

- Skips Probe/Classify phases
- Uses TPE from the start
- Each observation immediately informs next candidate
- Budget still respected

### When To Use

- ✅ Real-time control loops
- ✅ Single evaluations at a time
- ✅ External evaluation systems
- ✅ Streaming data sources

---

## Hybrid Approach

You can combine both modes:

```python
# Start with batch probing
for _ in range(3):
    batch = solver.ask()
    if batch:
        results = parallel_evaluate(batch)
        solver.tell(json.dumps(results))

# Switch to online refinement
while True:
    candidate = solver.ask_one()
    if candidate is None:
        break
    value = evaluate(candidate)
    solver.seed(json.dumps([{
        "params": candidate,
        "value": value,
        "cost": 1.0
    }]))
```

---

## Performance Implications

| Mode | Overhead/Candidate | Sample Efficiency |
|------|-------------------|-------------------|
| Batch | ~3ms | Higher (PCR) |
| Online | ~10ms | Lower (no Probe) |

**Guidance:**
- If evaluations < 10ms: Use **Batch** (amortize overhead)
- If evaluations > 100ms: Either works
- If real-time constraint: Use **Online**

---

## CLI Support

### Batch Mode
```bash
arqonhpo run --config config.json --script evaluate.sh
```

### Online Mode
```bash
# Interactive JSONL
arqonhpo interactive --config config.json

# Or via ask/tell commands
arqonhpo ask --config config.json --batch 1
# evaluate...
arqonhpo tell --state state.json --results result.json
```

---

## Next Steps

- [Quickstart](../quickstart.md) — Get running fast
- [Strategies](strategies.md) — Algorithm details
- [CLI Reference](../reference/cli.md) — Command documentation
