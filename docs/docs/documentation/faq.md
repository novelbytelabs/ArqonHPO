# FAQ

Frequently asked questions about ArqonHPO.

---

## General

### What is ArqonHPO?

ArqonHPO is a **microsecond-budget hyperparameter optimizer** written in Rust with Python bindings. It's designed for systems where optimization happens as a continuous control loop, not a one-time search.

### When should I use ArqonHPO vs Optuna?

| Use ArqonHPO when... | Use Optuna when... |
|---------------------|-------------------|
| Evaluations are <10ms | Evaluations are >1s |
| Need real-time tuning | Running offline experiments |
| Want deterministic replay | Need advanced samplers |
| Embedded/constrained systems | Rich visualization needed |

### Is it production-ready?

Yes. ArqonHPO is used in production for:
- LLM inference batch sizing
- Real-time control loops
- SRE automation

We have 91% test coverage and a strict Constitution.

---

## Algorithm

### What optimization algorithms does it use?

1. **Nelder-Mead** — Simplex method for smooth functions
2. **Multi-Start Nelder-Mead** — Parallel restarts for multimodal
3. **TPE** — Bayesian optimization for noisy landscapes

The Classify phase automatically selects the best strategy.

### Is it deterministic?

Yes! With a fixed seed, ArqonHPO produces identical results:

```python
# Same seed = same sequence
solver1 = ArqonSolver('{"seed": 42, ...}')
solver2 = ArqonSolver('{"seed": 42, ...}')
# solver1.ask() == solver2.ask()  ✓
```

### How does ArqonProbe work?

`ArqonProbe` generates **Low-Discrepancy Sequences** for uniform parameter space coverage:

```python
probe = ArqonProbe(config_json, seed=42)
point = probe.sample_at(0)    # First point
points = probe.sample_range(0, 100)  # Points 0-99
```

It's stateless and shardable — worker N can generate points [N*100, (N+1)*100) without coordination.

---

## Performance

### How fast is it?

| Metric | Value |
|--------|-------|
| Overhead per trial | ~3ms |
| Throughput | ~33,000 trials/sec |
| Memory | O(history_size) |

### Why is it faster than Optuna?

1. **Rust core** — No Python GIL, no interpreter overhead
2. **Batch processing** — Amortize overhead across many candidates
3. **Stateless probing** — No synchronization between workers

### Can it run on embedded systems?

Yes. The CLI compiles to a ~5MB static binary with no runtime dependencies.

---

## Safety

### What are Guardrails?

Guardrails prevent dangerous configurations:
- **Bounds** — Absolute limits on values
- **Delta limits** — Max change per update
- **Rate limits** — Max updates per second

See [Safety](concepts/safety.md) for details.

### Does it support rollback?

Yes. Configure a rollback policy:

```json
{
  "rollback_policy": {
    "max_consecutive_regressions": 3
  }
}
```

After 3 worse results, it reverts to the last good config.

---

## Integration

### How do I monitor it?

- **TUI** — `arqonhpo tui --state state.json`
- **Dashboard** — `arqonhpo dashboard --state state.json`
- **Prometheus** — `--metrics-addr 127.0.0.1:9898`

### Can I use it with Kubernetes?

Yes, via:
1. CLI in a sidecar container
2. Python bindings in your app
3. Dashboard for monitoring

Helm chart planned for v0.4.

### Does it work with Ray/Dask?

`ArqonProbe` is designed for distributed workers — each worker can sample independent ranges without coordination.

---

## Troubleshooting

### See [Troubleshooting Guide](troubleshooting.md)

---

## More Questions?

- [GitHub Discussions](https://github.com/novelbytelabs/ArqonHPO/discussions)
- [Issue Tracker](https://github.com/novelbytelabs/ArqonHPO/issues)
