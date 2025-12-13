# Spec: Probe-Gated Optimization (Two Use Cases)

**Project**: ArqonHPO (fresh repo)

**Status**: Draft

## Summary
Build an optimization engine that is explicitly designed for two common, high-value use cases:

1. **Fast simulation tuning (time-to-target)**: expensive objective evaluations (milliseconds to seconds) on mostly-smooth/structured landscapes where good candidates can be found quickly if the search is guided early.
2. **Sklearn-style model tuning (time-to-target)**: moderate-cost ML objectives where optimizer overhead matters and “good-enough quickly” beats “best eventually.”

The product is not trying to be universally SOTA across every search space; it is trying to be clearly competitive and measurably better for these two use cases.

## Problem Statement
General-purpose HPO (e.g., pure TPE) can be:
- too slow to reach a useful target when evaluations are expensive, and/or
- dominated by overhead when evaluations are moderate-cost.

We need a **deterministic, auditable, bounded-overhead** approach that performs a lightweight **probe → classify → choose refinement strategy** pipeline, optimized around time-to-target.

## Non-Negotiables (Constitution Alignment)
- **No bypass in default behavior**: runs must follow **probe → fixed-size classification → mode selection → refinement**.
- **Deterministic given seed**: same objective + bounds + seed + budget → identical decisions and identical best value (allowing only wall-clock timestamps to vary).
- **Bounded overhead**: per-eval policy work is O(1) and must not add extra objective calls.
- **Artifact auditability**: outputs are schema-versioned; include per-eval decisions, rewards/metrics, and environment fingerprint.
- **Canonical validation environment**: validation runs use `PYTHONNOUSERSITE=1` and the canonical env (internally: `helios-gpu-118`), without making that env name a customer-facing requirement.

## Definitions
- **Probe**: a deterministic, low-overhead sampler that produces an initial set of candidate evaluations.
- **Classification**: a fixed-size test that labels the landscape as suitable for structured refinement vs chaotic/rugged behavior.
- **Refinement**: the follow-on optimizer used for the remaining budget, chosen by the classification.
- **Time-to-target**: time (or eval count) to reach a specified objective threshold.

## User Scenarios (Prioritized)

### US1 (P0): Fast simulation tuning
**As a** user tuning simulator parameters,
**I want** to reach a useful threshold quickly,
**so that** I can iterate on designs without waiting for long runs.

**Acceptance scenarios**:
- **Given** bounds, seed, and budget, **when** I run the optimizer on an expensive smooth objective, **then** it reaches the target threshold faster than pure TPE in median time-to-target across a seed suite.
- **Given** the same inputs, **when** I rerun, **then** artifacts and best value match deterministically.

### US2 (P0): Sklearn-style ML tuning
**As a** user tuning a small ML model,
**I want** competitive time-to-target without large optimizer overhead,
**so that** model selection is fast and repeatable.

**Acceptance scenarios**:
- **Given** a sklearn objective and fixed seed/budget, **when** I run the optimizer, **then** its median time-to-target is competitive with (or better than) TPE at the chosen target thresholds.
- **Given** `PYTHONNOUSERSITE=1`, **when** I run benchmarks, **then** imports and results are stable (no user-site leakage).

## Functional Requirements

### FR1 — Probe phase
- Must evaluate a fixed number of probe samples derived deterministically from `seed`, `bounds`, and `probe_ratio`.
- Must record probe samples and their objective values in artifacts.

### FR2 — Classification phase
- Must evaluate a fixed-size classification test (default: 50 samples) and produce:
  - `variance_score` (or equivalent scalar),
  - `variance_label` in `{structured, chaotic}`,
  - a chosen `mode` for refinement.
- Classification must be authoritative for mode selection in default behavior.

### FR3 — Refinement modes
The optimizer must support these refinement strategies as first-class modes:
- **Structured mode**: a guided strategy that exploits the probe signal.
- **Chaotic mode**: a general-purpose strategy (e.g., TPE) suitable for rugged objectives.

### FR4 — Artifacts
- Must emit a schema-versioned run artifact containing:
  - inputs: bounds digest, budget, seed(s), probe_ratio,
  - phase timings,
  - classification outputs,
  - per-eval trace: phase, candidate, value, best-so-far, and any strategy decisions.
- Must capture an environment fingerprint sufficient to reproduce results.

### FR5 — Determinism
- All randomness must come from an explicit seeded RNG.
- Tie-breaking must be stable.

## Non-Goals
- Guarantee SOTA on all objective classes.
- Support for distributed/async evaluation in MVP.
- Support for arbitrary discrete/categorical-only spaces in MVP.

## Metrics & Evaluation Plan (What “Competitive” Means)
Primary metric for both use cases:
- **Median time-to-target** across a fixed seed suite.

Secondary metrics:
- Hit-rate (fraction of seeds reaching the target within budget).
- Best value at fixed budget.
- Overhead: optimizer CPU time per eval.

Baseline comparators:
- Random search.
- Pure Optuna TPE (or equivalent).

Minimum benchmark suite (MVP):
- **Fast sim tuning**: synthetic expensive smooth objectives (sleep-injected) and at least one “sim-like” structured function.
- **Sklearn tuning**: at least one linear model objective (SGDClassifier/Regressor) and one nonlinear model objective.

## Risks / Open Questions
- [NEEDS CLARIFICATION] Exact target thresholds per benchmark objective for time-to-target.
- [NEEDS CLARIFICATION] Definition of “structured” vs “chaotic” label threshold(s).
- [NEEDS CLARIFICATION] Which refinement strategies are in-scope for MVP (one per mode vs multiple).

## Deliverables (This Spec Only)
- This document defines what we are building; implementation and additional docs (plan/tasks) are intentionally deferred to SDD workflows.
