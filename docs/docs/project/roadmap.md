# Roadmap

ArqonHPO follows semantic versioning. Here's what's planned:

---

## Current: v0.3.x (Stable)

✅ **Released**

- PCR (Probe-Classify-Refine) algorithm
- Python bindings (ArqonSolver, ArqonProbe)
- CLI with TUI and Dashboard
- Safety Executor with Guardrails
- 91% test coverage

---

## Next: v0.4.0 (Q1 2026)

🔨 **In Development**

### Features
- [ ] Adaptive Nelder-Mead with restart detection
- [ ] Multi-objective optimization support
- [ ] Enhanced TPE with Scott's Rule bandwidth
- [ ] ArqonShip v1.0 (Self-Healing CI)

### Infrastructure
- [ ] Docker images for all platforms
- [ ] Helm chart for Kubernetes
- [ ] OpenTelemetry integration

---

## Future: v1.0.0 (Q2 2026)

🎯 **Planned**

### Features
- [ ] GPU-accelerated strategies (CUDA/Metal)
- [ ] Distributed optimization (sharded workers)
- [ ] Full Bayesian TPE
- [ ] REST API server mode

### Ecosystem
- [ ] Ray Tune integration
- [ ] MLflow tracking plugin
- [ ] Weights & Biases callback

---

## Long-term Vision

- **ArqonHPO as infrastructure** — As reliable as a database
- **Zero-configuration optimization** — Auto-strategy selection
- **Sub-microsecond overhead** — Usable in 1kHz control loops
- **Constitutional safety** — Auditable, reversible, bounded

---

## Request a Feature

Have an idea? Open a [Discussion](https://github.com/novelbytelabs/ArqonHPO/discussions) or [Issue](https://github.com/novelbytelabs/ArqonHPO/issues)!
