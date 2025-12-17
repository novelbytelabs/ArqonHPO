# Research: Adaptive Engine

**Feature**: 005-adaptive-engine  
**Date**: 2025-12-16  
**Status**: Complete

## Summary

This feature ports proven code from `origin/experiment/architecture-ideas`. All major design decisions are already validated by the ground truth implementation. This document records those decisions and rationale.

---

## Decisions

### 1. Atomic Config Implementation

**Decision**: Use `RwLock<Arc<ConfigSnapshot>>` for v1  
**Rationale**: Simpler to reason about; contention unlikely at intended update rates (≤10/sec)  
**Alternatives Considered**:
- `arc_swap::ArcSwap`: More complex, defer to v2 if benchmarks show contention
- Raw `AtomicPtr`: Unsafe, not worth the complexity

### 2. Parameter Representation

**Decision**: `ParamVec = SmallVec<[f64; 16]>` with `ParamRegistry` for name↔id mapping  
**Rationale**: Avoids HashMap in hot path; 16 params covers 99% of use cases without heap  
**Alternatives Considered**:
- `HashMap<String, f64>`: Too slow for microsecond budgets
- Fixed array `[f64; N]`: Inflexible for varying param counts

### 3. Lock-Free Audit Queue

**Decision**: `crossbeam::ArrayQueue` with fixed capacity  
**Rationale**: Battle-tested, O(1) push, bounded memory  
**Alternatives Considered**:
- `VecDeque` with Mutex: Blocking, unacceptable in hot path
- Custom SPSC queue: Unnecessary complexity

### 4. Objective Aggregation

**Decision**: `TrimmedMean(10%)` as default  
**Rationale**: Robust to outliers while preserving signal  
**Alternatives Considered**:
- `Mean`: Too sensitive to outliers
- `Median`: More robust but slower to compute

### 5. Evaluation Window

**Decision**: min(5 digests, 500ms timeout)  
**Rationale**: Balances responsiveness with noise rejection  
**Alternatives Considered**:
- Longer window (1s+): Too slow for real-time adaptation
- Shorter window (2 digests): Too noisy

### 6. Anti-Thrashing Rules

**Decision**: 
- `direction_flip_limit = 3` per dimension per minute
- `cooldown_after_flip_us = 30,000,000` (30s)
- `hysteresis_threshold = 0.1`

**Rationale**: Proven in POC-A (poc_a_knob_adaptation.py)  
**Alternatives Considered**:
- No anti-thrashing: Oscillation risk under noise
- Stricter limits: Over-constrains adaptation

### 7. Safe Mode Behavior

**Decision**: Latch that emits `NoChange` until explicit exit condition  
**Rationale**: Fail-safe; never auto-recover into potentially unstable state  
**Alternatives Considered**:
- Auto-recover after cooldown: Risk of repeated failures
- Full shutdown: Too aggressive

---

## Unknowns Resolved

| Unknown | Resolution |
|:---|:---|
| Config correlation method | Use `config_generation` field in TelemetryDigest |
| Staleness detection | Discard digests where `config_generation != expected` |
| Audit queue pressure | Enter SafeMode when queue is full; never silent drop |
| Homeostasis caching | Defer to post-005; contract defined in spec |

---

## External References

- **Ground Truth**: `origin/experiment/architecture-ideas:crates/core/src/adaptive_engine/`
- **POC Validation**: `docs/poc/arqon_poc_a_pocA/arqon_poc_a/poc_a_knob_adaptation.py`
- **Constitution**: v1.3.0 sections II.16-23, VIII.4-6, IX.2
