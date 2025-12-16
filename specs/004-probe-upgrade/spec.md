# Feature Specification: Probe Upgrade - Low-Discrepancy Sampling with Periodic Support

**Feature Branch**: `004-probe-upgrade`  
**Created**: 2025-12-16  
**Status**: Draft  
**Ground Truth**: `experiment/probe-upgrade` branch

---

## Constitution Constraints *(mandatory)*

Per constitution v1.1.0:
- Section II.12 (Probe Algorithm Specification) — Kronecker/Weyl required, p/1000 banned
- Section II.13 (Dimension Type Contract) — Linear, Log, Periodic with circular arithmetic
- Section II.14 (Multi-Start Strategy Contract) — K-parallel with farthest-point seeding
- Section II.15 (Parallel Sharding Contract) — Stateless, collision-free, SDK parity
- Section IV.5 (Probe Guardrail Tests) — 6 mandatory test classes
- Section VIII.3 (Time-to-Target Metrics) — Evals-to-Threshold, Hit-by-N, Median-Best
- Section XI.3 (Benchmark Schema Contract) — Objective suite, cost regimes, output schema
- Section XI.4 (SDK Binding Compliance) — Determinism parity, sharding verification

---

## Overview

Upgrade ArqonHPO's probe algorithm from the flawed p/1000 heuristic to a mathematically rigorous Kronecker/Weyl sequence, with support for periodic dimensions, stateless parallel sharding, and multi-start NM diversity.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Low-Discrepancy Probe (Priority: P1)

As a user running optimization, I want deterministic low-discrepancy samples so that I avoid collisions and striping artifacts that waste budget.

**Why this priority**: The probe is the first phase of every run. A flawed probe corrupts all downstream optimization.

**Independent Test**: Can be fully tested by generating samples and measuring discrepancy metrics without running full optimization.

**Acceptance Scenarios**:

1. **Given** a 2D search space with 40-sample budget, **When** I probe, **Then** there are zero duplicate points.
2. **Given** any seed, **When** I probe K samples, **Then** quality does not depend on total N (anytime property).
3. **Given** high-dimensional space (10D), **When** I probe, **Then** no striping artifacts appear in projections.

---

### User Story 2 - Periodic Dimension Support (Priority: P2)

As a user optimizing angles or phases, I want circular/toroidal arithmetic so that the optimizer respects the wrap-around topology.

**Why this priority**: Periodic dimensions are common in robotics, physics, and phase-based optimization.

**Independent Test**: Can test NM operations on periodic bounds and verify wrap-around behavior.

**Acceptance Scenarios**:

1. **Given** a Periodic dimension [0, 2π), **When** NM reflects past 0, **Then** it wraps to 2π (not clips).
2. **Given** Periodic bounds, **When** probe samples, **Then** edge bias is eliminated.
3. **Given** all-periodic dimensions, **When** computing centroid, **Then** circular_mean01 is used.

---

### User Story 3 - Stateless Parallel Sharding (Priority: P2)

As a user running multi-worker optimization, I want stateless probe sharding so that workers produce disjoint samples without coordination.

**Why this priority**: Parallel evaluation is critical for expensive objectives.

**Independent Test**: Run multiple workers with disjoint index ranges and verify hash collision-free.

**Acceptance Scenarios**:

1. **Given** 4 workers with disjoint index ranges, **When** all probes run, **Then** samples are collision-free.
2. **Given** same (seed, index) on any worker count, **Then** sample is identical.
3. **Given** Python ArqonProbe, **When** called with same config, **Then** results match Rust core.

---

### User Story 4 - Multi-Start NM Diversity (Priority: P3)

As a user running NM refinement, I want farthest-point seeding so that K parallel starts explore diverse regions.

**Why this priority**: Single-start NM gets stuck in local minima on multimodal landscapes.

**Independent Test**: Verify farthest-point selection produces maximally spread starts.

**Acceptance Scenarios**:

1. **Given** K=4 starts, **When** seeding from top candidates, **Then** starts are maximally spread.
2. **Given** stall threshold of 10, **When** a start stalls, **Then** rotation to next start occurs.
3. **Given** triage budget exhausted, **When** evaluating, **Then** commit to best start.

---

### User Story 5 - CP Shift Randomization (Priority: P3)

As a user needing stochastic exploration, I want Cranley-Patterson shifts so that I get randomized QMC without breaking determinism.

**Why this priority**: CP shifts improve coverage on noisy landscapes while maintaining reproducibility.

**Independent Test**: Verify shifts are applied uniformly and same seed produces identical results.

**Acceptance Scenarios**:

1. **Given** a global shift vector, **When** probing, **Then** all samples shift uniformly in [0,1).
2. **Given** same seed, **When** run twice, **Then** results are identical.
3. **Given** no cp_shift config, **When** probing, **Then** auto-generate from seed.

---

### Edge Cases

- **Zero dimensions**: Error "At least one dimension required"
- **Degenerate bounds** (min == max): Collapse to constant
- **Periodic span > 360°**: Normalize to one period
- **Invalid spice ratio** (> 1.0): Clamp to [0.0, 1.0]
- **K starts > budget**: Error "Insufficient budget for K starts"
- **All periodic dimensions**: Full toroidal topology applied

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Probe MUST use Kronecker/Weyl sequences with irrational slopes (prime square roots).
- **FR-002**: The p/1000 heuristic is BANNED.
- **FR-003**: Probe MUST be anytime (quality of K samples independent of N).
- **FR-004**: Periodic dimensions MUST use circular arithmetic (wrap01, diff01, circular_mean01).
- **FR-005**: Log dimensions MUST use log-space transform → operate → inverse.
- **FR-006**: NM on Periodic MUST use circular reflection/expansion/contraction.
- **FR-007**: Sharding MUST produce identical samples for (seed, index) regardless of worker count.
- **FR-008**: Multi-start MUST use farthest-point seeding; clustered seeding is BANNED.
- **FR-009**: Stall detection MUST trigger start rotation; unbounded stalling is BANNED.
- **FR-010**: Random spice ratio MUST be configurable (default 10%).

### Configuration Schema

| Field | Type | Default | Description |
|:---|:---|:---|:---|
| `probe_type` | "prime_sqrt" \| "legacy" | "prime_sqrt" | Probe algorithm selection |
| `random_spice_ratio` | f64 [0.0, 1.0] | 0.1 | Fraction of uniform random points |
| `cp_shift` | Option<Vec<f64>> | None (auto) | Cranley-Patterson shift vector |
| `multi_start_k` | usize | 4 | Number of parallel NM starts |
| `triage_budget` | usize | 20 | Evals per start before commitment |
| `stall_threshold` | usize | 10 | Iters without improvement before rotation |

### Key Entities

- **PrimeSqrtSlopesRotProbe**: Kronecker sequence probe with CP shift support
- **PrimeSqrtSlopesRotConfig**: Configuration (prime_offset, rot_offset, rot_alpha, spice)
- **Scale::Periodic**: Dimension type with toroidal topology
- **MultiStartNM**: K-parallel NM strategy with farthest-point seeding
- **ArqonProbe**: Python binding exposing stateless sharding

---

## Non-Goals (Explicit Exclusions)

- **NOT** implementing CMA-ES (future work)
- **NOT** wiring classifier routing in this feature (separate spec)
- **NOT** matching Optuna TPE exactly (separate spec)
- **NOT** changing the Classify phase (only Probe and Refine)
- **NOT** adding new benchmark objectives (use existing suite)

---

## Migration & Compatibility

- Legacy `UniformProbe` and `PrimeIndexProbe` remain available but deprecated.
- Config field `probe_type: "prime_sqrt"` selects new probe (default in v2).
- Old configs without `probe_type` fall back to legacy for one version, then error.
- Python binding `ArqonProbe` is NEW; existing `ArqonSolver` unchanged.

---

## Performance Budget

- Probe point generation: O(1) per point
- Probe latency: < 10µs per sample (p99)
- No heap allocations in hot path
- Multi-start overhead: < 1ms for K=4

---

## Observability

- Log probe algorithm selection at INFO level
- Include `probe_type`, `spice_ratio`, `cp_shift` in run artifacts
- Trace sharding index ranges and collision stats (DEBUG)
- Report multi-start rotation events in audit log

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: New probe beats legacy by ≥60% on shifted instances (TestProbeOnlyQuality)
- **SC-002**: NM wins on structured landscapes—Sphere, Rosenbrock (TestStructuredRouting)
- **SC-003**: Probe is robust on Rastrigin (≥ Random baseline) (TestMultimodalGuardrail)
- **SC-004**: Probe geometry is deterministic across runs (TestGeometryRegression)
- **SC-005**: NM periodic arithmetic is correct (TestStructuredNMCorrectness)
- **SC-006**: Time-to-target metrics are computed and reported (TestTimeToQuality)
- **SC-007**: All 6 guardrail test classes pass in CI
- **SC-008**: Sharding hash matches single vs multi-worker (SDK Parity)
