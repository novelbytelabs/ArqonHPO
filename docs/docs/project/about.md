# About ArqonHPO

ArqonHPO is developed by **NovelByte Labs**.

---

## Mission

**Make optimization as reliable as infrastructure.**

Traditional HPO libraries treat optimization as an afterthought — something you "pip install, run for 3 days, hope for the best."

We reject this. ArqonHPO is built from the ground up for production:

- **Microsecond overhead** — Use it in control loops
- **Deterministic execution** — Reproduce any result
- **Constitutional safety** — Every update through guardrails
- **Observable** — Know exactly what's happening

---

## The Philosophy

> "Optimization is infrastructure. It should be as reliable as a database and as fast as a cache lookup."

### Principles

1. **Speed is a feature** — <3ms overhead means real-time is possible
2. **Safety is non-negotiable** — Guardrails, rollback, audit trail
3. **Determinism is default** — Same seed = same sequence
4. **Observability is built-in** — Not an afterthought

---

## The Constitution

This project operates under a strict [Constitution](constitution.md) that mandates:

| Principle                 | Meaning                              |
| ------------------------- | ------------------------------------ |
| **No Happy Path Testing** | Every edge case must be tested       |
| **No Silent Failures**    | All errors must surface with context |
| **Zero Unbounded Growth** | Memory/CPU must be bounded           |
| **Audit Everything**      | Every state change is logged         |

Values are enforced by CI, not just written in docs.

---

## History

| Version | Date    | Milestone                         |
| ------- | ------- | --------------------------------- |
| v0.1    | 2025-06 | Initial release, Nelder-Mead only |
| v0.2    | 2025-09 | PCR algorithm, Python bindings    |
| v0.3    | 2026-01 | Safety Executor, TPE, Dashboard   |
| v0.4    | 2026-Q1 | (Planned) Helm, OTel, Docker      |
| v1.0    | 2026-Q2 | (Planned) GPU, distributed        |

---

## Team

ArqonHPO is developed by NovelByte Labs with contributions from the open source community.

---

## Next Steps

- [Roadmap](roadmap.md) — What's coming
- [Constitution](constitution.md) — Our principles
- [Contributing](CONTRIBUTING.md) — How to help
