# v0.2.0 - Adaptive Engine & Hot-Path Enforcement

**Release Date**: 2025-12-17

## 🚀 Highlights

This release introduces the **Adaptive Engine** - a real-time parameter tuning system with microsecond latency, plus strict **Hot-Path Enforcement** mandated by the ArqonHPO Constitution.

### ⚡ Adaptive Engine

The new `hotpath` crate provides:

- **SPSA Optimizer** - Simultaneous Perturbation Stochastic Approximation with ±1 Bernoulli perturbations
- **Safety Executor** - Tier 1 guardrails preventing unbounded changes, with rollback support
- **Control Safety** - Anti-thrashing detection, direction flip limits, delta budget tracking
- **Audit Queue** - Lock-free, non-blocking event logging with guaranteed drop counting
- **Telemetry Ingestion** - Ring buffer for digest collection

### 🛡️ Hot-Path Enforcement (Constitution VIII.3)

- `HashMap` **banned** in Tier 1/2 code via `clippy.toml`
- Dense vectors (`Vec`, `ParamVec`) mandated for O(1) access
- Strict `#![deny(clippy::disallowed_types)]` in `hotpath` crate
- Determinism verification test for bit-for-bit replay

### 📊 Performance

| Benchmark                     | Latency (p50) |
| ----------------------------- | ------------- |
| T1 Apply (4 params)           | 117 ns        |
| T2 Decision (4 params)        | 209 ns        |
| SPSA Perturbation (16 params) | 176 ns        |
| Audit Enqueue                 | 35 ns         |
| Telemetry Push                | 13 ns         |

All timings are **4 orders of magnitude** below Constitution VIII.4 targets (T1 ≤ 100µs, T2 ≤ 1000µs).

## 📦 New Crates

- **`hotpath`** - Strict hot-path crate with Constitution-mandated invariants

## 📁 New Files

### Core

- `crates/hotpath/` - Complete Adaptive Engine implementation
- `clippy.toml` - Repo-root HashMap ban
- `crates/core/benches/adaptive_engine_latency.rs` - Latency benchmarks
- `crates/core/tests/artifact_contract.rs` - Artifact schema verification

### Specifications

- `specs/005-adaptive-engine/` - Full spec, plan, tasks, contracts
- `specs/004-probe-upgrade/` - PCR probe improvements

### Constitution

- Updated to **v1.4.1** with VIII.3 Hot-Path Enforcement rules

## ⬆️ Upgrade Notes

No breaking changes for Python users. The new `hotpath` crate is additive and will be exposed via PyO3 bindings in a future release.

## 🙏 Acknowledgments

Built with Constitution-driven development using [SpecKit](https://github.com/humanloop/speckit).

---

**Full Changelog**: https://github.com/novelbytelabs/ArqonHPO/compare/v0.1.0...v0.2.0
