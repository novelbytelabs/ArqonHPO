# Implementation Plan: Probe Upgrade

**Branch**: `004-probe-upgrade` | **Date**: 2025-12-16 | **Spec**: [spec.md](spec.md)  
**Ground Truth**: `experiment/probe-upgrade` branch (code exists, this plan ports it cleanly)

## Summary

Upgrade ArqonHPO's probe algorithm from the flawed p/1000 heuristic to a Kronecker/Weyl sequence with periodic dimension support, stateless parallel sharding, and multi-start NM diversity. The implementation already exists in `experiment/probe-upgrade`—this plan defines the clean port to `feature/probe` with full SDD compliance.

---

## Technical Context

**Language/Version**: Rust 1.75+, Python 3.10+ (bindings)  
**Primary Dependencies**: `rand_chacha` (RNG), `serde` (serialization), `pyo3` (Python bindings)  
**Storage**: N/A (in-memory, artifacts to disk)  
**Testing**: `cargo test`, `pytest` (Python bindings), `benchmarks/test_probe_guardrails.py`  
**Target Platform**: Linux server, macOS, Windows (cross-platform Rust)  
**Project Type**: Library crate with CLI and Python bindings  
**Performance Goals**: < 10µs per probe sample (p99), O(1) per point  
**Constraints**: No heap allocations in hot path, < 1ms multi-start overhead  
**Scale/Scope**: 10D search spaces, 1000+ sample budgets, K=4 parallel starts

---

## Constitution Check

*GATE: All items verified against constitution v1.1.0*

| Check | Status | Evidence |
|:---|:---|:---|
| Probe uses Kronecker/Weyl (II.12) | ✅ | `PrimeSqrtSlopesRotProbe` in ground truth |
| p/1000 banned (II.12) | ✅ | Legacy probe deprecated, not used |
| Periodic uses circular arithmetic (II.13) | ✅ | `wrap01`, `diff01`, `circular_mean01` in config.rs |
| Multi-start uses farthest-point (II.14) | ✅ | `MultiStartNM` in strategies/ |
| Sharding is stateless (II.15) | ✅ | `ArqonProbe` in Python bindings |
| Guardrail tests exist (IV.5) | ✅ | `benchmarks/test_probe_guardrails.py` |
| Time-to-target metrics (VIII.3) | ✅ | `TestTimeToQuality` class |
| SDK parity verified (XI.4) | ✅ | Hash comparison in sharding tests |

**Result**: PASS — proceed to implementation.

---

## Project Structure

### Documentation (this feature)

```text
specs/004-probe-upgrade/
├── spec.md              # Feature specification (done)
├── plan.md              # This file (done)
├── checklists/          # Quality checklists
└── tasks.md             # Implementation tasks (next)
```

### Source Code (from ground truth)

```text
crates/core/src/
├── probe.rs             # [MODIFY] Add PrimeSqrtSlopesRotProbe
├── config.rs            # [MODIFY] Add Scale::Periodic, circular helpers
├── machine.rs           # [MODIFY] Wire new probe as default
└── strategies/
    ├── mod.rs           # [MODIFY] Export multi_start_nm
    ├── nelder_mead.rs   # [MODIFY] Add periodic arithmetic
    └── multi_start_nm.rs # [NEW] K-parallel NM with farthest-point

bindings/python/
├── src/lib.rs           # [MODIFY] Add ArqonProbe class
└── python/arqonhpo/__init__.py  # [MODIFY] Export ArqonProbe

benchmarks/
├── test_probe_guardrails.py  # [NEW] CI guardrail tests (copy from ground truth)
└── reporting/           # [NEW] Phase 8 benchmark reporting
```

**Structure Decision**: Single library crate + CLI + Python bindings (existing structure).

---

## Implementation Strategy

Since the code exists in `experiment/probe-upgrade`, the implementation strategy is:

### Phase 1: Core Probe Algorithm
1. Cherry-pick `PrimeSqrtSlopesRotProbe` from ground truth
2. Cherry-pick `Scale::Periodic` and circular helpers from `config.rs`
3. Wire new probe as default in `machine.rs`
4. Verify: `cargo test`

### Phase 2: Periodic Dimension Support
1. Cherry-pick NM periodic arithmetic from `nelder_mead.rs`
2. Verify: `TestStructuredNMCorrectness` passes

### Phase 3: Multi-Start NM
1. Cherry-pick `multi_start_nm.rs` from ground truth
2. Wire into strategy selection
3. Verify: Farthest-point seeding produces diverse starts

### Phase 4: Parallel Sharding & Python Bindings
1. Cherry-pick `ArqonProbe` from `bindings/python/src/lib.rs`
2. Update `__init__.py` exports
3. Verify: Hash comparison matches single vs multi-worker

### Phase 5: CI Guardrail Tests
1. Copy `benchmarks/test_probe_guardrails.py` from ground truth
2. Run full suite: `pytest benchmarks/test_probe_guardrails.py -v`
3. Verify: All 6 test classes pass

---

## Files Changed Summary

| File | Action | Description |
|:---|:---|:---|
| `crates/core/src/probe.rs` | MODIFY | Add PrimeSqrtSlopesRotProbe (~300 lines) |
| `crates/core/src/config.rs` | MODIFY | Add Scale::Periodic, wrap01, diff01, circular_mean01 (~60 lines) |
| `crates/core/src/machine.rs` | MODIFY | Wire new probe as default |
| `crates/core/src/strategies/nelder_mead.rs` | MODIFY | Add periodic arithmetic |
| `crates/core/src/strategies/multi_start_nm.rs` | NEW | K-parallel NM strategy (~330 lines) |
| `crates/core/src/strategies/mod.rs` | MODIFY | Export multi_start_nm |
| `bindings/python/src/lib.rs` | MODIFY | Add ArqonProbe class |
| `bindings/python/python/arqonhpo/__init__.py` | MODIFY | Export ArqonProbe |
| `benchmarks/test_probe_guardrails.py` | NEW | CI guardrail tests (~720 lines) |

---

## Verification Plan

### Automated Tests
```bash
# Unit tests (Rust)
cargo test --package arqonhpo-core

# Guardrail tests (Python)
conda run -n helios-gpu-118 pytest benchmarks/test_probe_guardrails.py -v

# Sharding parity test
conda run -n helios-gpu-118 python benchmarks/demo_parallel_sharding.py
```

### Success Criteria Verification
| Criterion | Test | Command |
|:---|:---|:---|
| SC-001: Probe beats legacy ≥60% | TestProbeOnlyQuality | `pytest -k TestProbeOnlyQuality` |
| SC-002: NM wins on structured | TestStructuredRouting | `pytest -k TestStructuredRouting` |
| SC-003: Robust on Rastrigin | TestMultimodalGuardrail | `pytest -k TestMultimodalGuardrail` |
| SC-004: Geometry deterministic | TestGeometryRegression | `pytest -k TestGeometryRegression` |
| SC-005: NM periodic correct | TestStructuredNMCorrectness | `pytest -k TestStructuredNMCorrectness` |
| SC-006: Time-to-target reported | TestTimeToQuality | `pytest -k TestTimeToQuality` |

---

## Complexity Tracking

No constitution violations requiring justification. All changes align with v1.1.0 amendments.
