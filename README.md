# ArqonHPO

<!-- CI / Build Status -->
[![CI](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml/badge.svg)](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml)
[![Linux](https://img.shields.io/badge/Linux-passing-brightgreen?logo=linux&logoColor=white)](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml)
[![macOS](https://img.shields.io/badge/macOS-passing-brightgreen?logo=apple&logoColor=white)](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml)
[![Windows](https://img.shields.io/badge/Windows-passing-brightgreen?logo=windows&logoColor=white)](https://github.com/novelbytelabs/ArqonHPO/actions/workflows/ci.yml)

<!-- Coverage & Quality -->
<!-- Coverage (Dynamic Components) -->
[![Project](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/novelbytelabs/ArqonHPO/badges/project.json)](https://github.com/novelbytelabs/ArqonHPO/actions)
[![Core](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/novelbytelabs/ArqonHPO/badges/core.json)](https://github.com/novelbytelabs/ArqonHPO/tree/main/crates/core)
[![Hotpath](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/novelbytelabs/ArqonHPO/badges/hotpath.json)](https://github.com/novelbytelabs/ArqonHPO/tree/main/crates/hotpath)
[![CLI](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/novelbytelabs/ArqonHPO/badges/cli.json)](https://github.com/novelbytelabs/ArqonHPO/tree/main/crates/cli)
<!-- Hybrid Codecov link -->
[![codecov](https://codecov.io/gh/novelbytelabs/ArqonHPO/branch/main/graph/badge.svg)](https://codecov.io/gh/novelbytelabs/ArqonHPO)

<!-- Version & Language -->
[![Crates.io](https://img.shields.io/crates/v/arqonhpo-core?label=crates.io)](https://crates.io/crates/arqonhpo-core)
[![PyPI](https://img.shields.io/pypi/v/arqonhpo?label=PyPI)](https://pypi.org/project/arqonhpo/)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.10%2B-blue?logo=python&logoColor=white)](https://www.python.org/)

<!-- Documentation & License -->
[![Docs](https://img.shields.io/badge/docs-mkdocs-blue?logo=readthedocs)](https://novelbytelabs.github.io/ArqonHPO/)
[![License](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

<!-- GitHub Stats -->
[![GitHub stars](https://img.shields.io/github/stars/novelbytelabs/ArqonHPO?style=social)](https://github.com/novelbytelabs/ArqonHPO/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/novelbytelabs/ArqonHPO?style=social)](https://github.com/novelbytelabs/ArqonHPO/network/members)
[![Contributors](https://img.shields.io/github/contributors/novelbytelabs/ArqonHPO)](https://github.com/novelbytelabs/ArqonHPO/graphs/contributors)

<!-- Downloads -->
[![Downloads](https://img.shields.io/github/downloads/novelbytelabs/ArqonHPO/total?label=downloads)](https://github.com/novelbytelabs/ArqonHPO/releases)
[![PyPI Downloads](https://img.shields.io/pypi/dm/arqonhpo?label=PyPI%20downloads)](https://pypi.org/project/arqonhpo/)


# **Machine-speed optimization for live systems.**  

ArqonHPO is a Rust-first optimization runtime that can sit *inside* a control loop: it proposes bounded parameter updates, ingests telemetry/reward signals, and produces deterministic, auditable decisions with sub-microsecond overhead.

It’s not “run an offline study.” It’s: **measure → decide → apply → measure again**.

---

## What ArqonHPO is for

ArqonHPO shines when:

- Your system has **knobs** (timeouts, batch sizes, thresholds, weights, cache TTLs, controller gains…)
- You have **telemetry or a reward signal** (latency, error rate, cost/request, quality proxy…)
- The “best settings” **move over time** (drift, load changes, hardware changes, data changes)
- Human tuning is expensive and slow, and “set it once” goes stale

If your evaluation is extremely expensive (minutes/hours per trial), classic offline HPO can be a better fit.  
If your loop must run at machine speed, ArqonHPO is built for that.

---

## Core idea (one sentence)

**Optuna optimizes experiments. ArqonHPO optimizes decisions.**

---

## What’s in the box

### Tier-2: Adaptive decision engine (hot path)
- Online optimizer designed for **continuous tuning under drift**
- Runs under strict time budgets
- Produces **bounded, stable deltas** rather than “wild” parameter jumps 

### Tier-1: Safety executor (hot path)
- Guardrails: bounds, max-delta, cooldown/dwell, rollback hooks
- Non-blocking audit + telemetry emission (never blocks the hot path) 

### Determinism + evidence
- Stable parameter ordering (registry) for replayability
- Seeded decisioning, audit trail, and artifacts for “why did it change?” 

---

## Performance

ArqonHPO is designed to add **near-zero overhead** to a live loop.

Typical p50 numbers (example from local benches):

- **T2 (decide/observe): ~200–235 ns**
- **T1 (apply): ~110–120 ns**

Your actual performance will depend on CPU, build flags, and integration, but the design goal is consistent:
**fast enough to be in the loop, safe enough to ship.**

---

## Installation

```bash
pip install arqonhpo
```

Build from source (Python bindings via maturin):

```bash
git clone https://github.com/novelbytelabs/ArqonHPO.git
cd ArqonHPO
pip install maturin
maturin develop -m bindings/python/Cargo.toml
```

---

## Quick start (ask/tell loop)

> This example shows the *shape* of integration: you supply measurements, ArqonHPO returns proposals.

```python
import json
from arqonhpo import ArqonSolver

def objective(params):
    x, y = params["x"], params["y"]
    return (x - 2)**2 + (y + 1)**2

config = {
    "seed": 42,
    "budget": 200,
    "bounds": {
        "x": {"min": -10.0, "max": 10.0},
        "y": {"min": -10.0, "max": 10.0}
    },
    # Optional production-style safety knobs (examples):
    # "max_delta": {"x": 0.25, "y": 0.25},
    # "cooldown_steps": 5,
}

solver = ArqonSolver(json.dumps(config))

best = {"value": float("inf"), "params": None}

while (batch := solver.ask()) is not None:
    results = []
    for params in batch:
        value = objective(params)
        if value < best["value"]:
            best = {"params": params, "value": value}
        results.append({"params": params, "value": value, "cost": 1.0})
    solver.tell(json.dumps(results))

print(f"Best: {best['params']} -> {best['value']:.4f}")
```

---

## How it fits in a real system

ArqonHPO expects three things:

1. **Telemetry / reward signal**
   A scalar “how are we doing?” (latency, error rate, cost, quality proxy, etc.)

2. **Actuation surface**
   Parameters you can safely change at runtime

3. **Policy / constraints**
   Bounds + limits that make changes safe and reversible

ArqonHPO does **not** require you to adopt a giant platform.
But to deliver the “runtime optimization” promise, you will typically integrate at least:

* a telemetry digest (even minimal),
* safe actuation,
* guardrails (policy).

---

## Constitution (merge-blocking invariants)

ArqonHPO is developed under a Constitution: a living spec of invariants and timing contracts.

Highlights:

* **Hot-Path Representation:** Tier-1/Tier-2 code uses dense parameter vectors (no `HashMap` in hot path)
* **Timing Contracts:** T1/T2 must remain within defined budgets (p99 targets)
* **Non-blocking observability:** audit/telemetry must never block the hot path
* **Deterministic replay:** decisions must be reproducible from artifacts + seeds

See: `project/constitution.md`

---

## Documentation

- [**Quickstart**](https://novelbytelabs.github.io/ArqonHPO/documentation/quickstart/)
- [**Cookbook (Recipes)**](https://novelbytelabs.github.io/ArqonHPO/documentation/cookbook/)
- [**API Reference**](https://novelbytelabs.github.io/ArqonHPO/documentation/reference/python/)
- [**Architecture**](https://novelbytelabs.github.io/ArqonHPO/why/architecture/)

---

## Contributing

See `project/CONTRIBUTING.md`.

---

## License

Apache License 2.0 — see `project/LICENSE`.

