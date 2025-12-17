# Tasks: Adaptive Engine

**Feature**: 005-adaptive-engine  
**Input**: Design documents from `/specs/005-adaptive-engine/`  
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/ ✓

**Tests**: REQUIRED (TDD) — ArqonHPO solver logic and sampling math (Constitution IV.6)

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- All paths relative to `crates/core/src/`

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Module structure and dependencies

- [x] T001 Create adaptive_engine/ module directory under crates/core/src/
- [x] T002 Add dependencies to Cargo.toml: smallvec, crossbeam-queue
- [x] T003 [P] Create mod.rs with module declarations and re-exports
- [x] T004 [P] Add `pub mod adaptive_engine` to crates/core/src/lib.rs

**Checkpoint**: Module skeleton compiles with `cargo check` ✅

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types that ALL user stories depend on. MUST complete before user story work.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Core Types

- [x] T005 Create ParamId type alias (u16) in adaptive_engine/config_atomic.rs
- [x] T006 [P] Create ParamVec type (SmallVec<[f64; 16]>) in adaptive_engine/config_atomic.rs
- [x] T007 [P] Create ParamRegistry struct with name↔id mapping in adaptive_engine/config_atomic.rs
- [x] T008 Create ConfigSnapshot struct with params: ParamVec and generation: u64
- [x] T009 Create AtomicConfig with RwLock<Arc<ConfigSnapshot>> and AtomicU64 generation

### Tests for Foundational

- [x] T010 [P] Unit test: ParamVec size assertion (≤256 bytes) in tests/test_adaptive_engine.rs
- [x] T011 [P] Unit test: ConfigSnapshot generation monotonicity in tests/test_adaptive_engine.rs
- [x] T012 [P] Unit test: AtomicConfig::swap() increments generation in tests/test_adaptive_engine.rs
- [x] T013 Unit test: AtomicConfig is Send + Sync (compile-time check)

**Checkpoint**: Foundation types compile and tests pass ✅

---

## Phase 3: User Story 1 - Online Parameter Adaptation (Priority: P1) 🎯 MVP

**Goal**: Engine automatically adapts parameters as traffic patterns change

**Independent Test**: Deploy with synthetic drift; verify config converges toward optimum

### Tests for US1 (Write FIRST, ensure they FAIL)

- [ ] T014 [P] [US1] Test: SPSA produces identical perturbations for same (seed, k) — tests/test_adaptive_engine.rs
- [ ] T015 [P] [US1] Test: SPSA returns proposal after exactly 2 eval windows — tests/test_adaptive_engine.rs
- [ ] T016 [P] [US1] Test: TelemetryDigest ≤128 bytes (const_assert) — tests/test_adaptive_engine.rs
- [ ] T017 [P] [US1] Integration test: config converges under stationary objective — tests/test_adaptive_engine.rs

### Implementation for US1

- [ ] T018 [P] [US1] Create TelemetryDigest struct with config_generation in adaptive_engine/telemetry.rs
- [ ] T019 [P] [US1] Create TelemetryRingBuffer with fixed-capacity Box<[Option<T>]> in adaptive_engine/telemetry.rs
- [ ] T020 [US1] Implement DigestValidity enum and validate_digest() in adaptive_engine/telemetry.rs
- [ ] T021 [US1] Create SpsaState enum (Ready, ApplyingPlus, WaitingPlus, etc.) in adaptive_engine/spsa.rs
- [ ] T022 [US1] Implement Spsa struct with ChaCha8Rng in adaptive_engine/spsa.rs
- [ ] T023 [US1] Implement SPSA perturbation generation (±1 Bernoulli) in adaptive_engine/spsa.rs
- [ ] T024 [US1] Implement SPSA step() with state machine transitions in adaptive_engine/spsa.rs
- [ ] T025 [US1] Implement eval window collection and objective aggregation (TrimmedMean) in adaptive_engine/spsa.rs
- [ ] T026 [US1] Create Proposal enum (ApplyPlus, ApplyMinus, Update, NoChange) in adaptive_engine/proposer.rs
- [ ] T027 [US1] Create AdaptiveProposer trait in adaptive_engine/proposer.rs
- [ ] T028 [US1] Implement AdaptiveProposer for Spsa in adaptive_engine/proposer.rs
- [ ] T029 [US1] Create SafeExecutor trait in adaptive_engine/executor.rs

**Checkpoint**: SPSA optimizer works, convergence test passes

---

## Phase 4: User Story 2 - Safety-First Operation (Priority: P1)

**Goal**: Adaptation never destabilizes production, even under adversarial telemetry

**Independent Test**: Inject adversarial telemetry; verify engine enters SafeMode

### Tests for US2 (Write FIRST, ensure they FAIL)

- [ ] T030 [P] [US2] Test: SafetyExecutor rejects delta > max_delta — tests/test_adaptive_engine.rs
- [ ] T031 [P] [US2] Test: SafetyExecutor rejects out-of-bounds — tests/test_adaptive_engine.rs
- [ ] T032 [P] [US2] Test: SafetyExecutor rejects unknown parameter — tests/test_adaptive_engine.rs
- [ ] T033 [P] [US2] Test: Stale digests (wrong config_generation) are discarded — tests/test_adaptive_engine.rs
- [ ] T034 [P] [US2] Test: Rollback restores baseline exactly — tests/test_adaptive_engine.rs

### Implementation for US2

- [ ] T035 [P] [US2] Create Violation enum (9 variants) in adaptive_engine/executor.rs
- [ ] T036 [P] [US2] Create Guardrails struct with defaults in adaptive_engine/executor.rs
- [ ] T037 [US2] Implement SafetyExecutor struct with validate_delta() in adaptive_engine/executor.rs
- [ ] T038 [US2] Implement clamp_to_bounds() in SafetyExecutor
- [ ] T039 [US2] Implement set_baseline() and rollback() in SafetyExecutor
- [ ] T040 [US2] Implement rate limiting with window tracking in SafetyExecutor
- [ ] T041 [US2] Implement SafeExecutor trait for SafetyExecutor in adaptive_engine/executor.rs
- [ ] T042 [US2] Add staleness check to telemetry validation in adaptive_engine/telemetry.rs

**Checkpoint**: All guardrail tests pass, rollback works

---

## Phase 5: User Story 3 - Anti-Thrashing Under Noise (Priority: P2)

**Goal**: Engine doesn't oscillate parameters under noisy objectives

**Independent Test**: Inject oscillating objective; verify direction flip count bounded

### Tests for US3 (Write FIRST, ensure they FAIL)

- [ ] T043 [P] [US3] Test: Direction flips ≤3 per dimension per minute — tests/test_adaptive_engine.rs
- [ ] T044 [P] [US3] Test: Cooldown enforced after flip limit exceeded — tests/test_adaptive_engine.rs
- [ ] T045 [P] [US3] Test: SafeMode entered after regression_count_limit exceeded — tests/test_adaptive_engine.rs
- [ ] T046 [P] [US3] Test: Constraint-first — proposals restore feasibility when margin < 0 — tests/test_adaptive_engine.rs

### Implementation for US3

- [ ] T047 [P] [US3] Create SafeMode struct and SafeModeReason/SafeModeExit enums in adaptive_engine/control_safety.rs
- [ ] T048 [US3] Implement DirectionTracker for anti-thrashing in adaptive_engine/control_safety.rs
- [ ] T049 [US3] Implement cumulative delta budget tracking in adaptive_engine/control_safety.rs
- [ ] T050 [US3] Implement regression detector (consecutive worsening) in adaptive_engine/control_safety.rs
- [ ] T051 [US3] Implement constraint-first proposal filtering in adaptive_engine/control_safety.rs
- [ ] T052 [US3] Implement SafeMode latch with exit conditions in adaptive_engine/control_safety.rs
- [ ] T053 [US3] Integrate control_safety into SafetyExecutor in adaptive_engine/executor.rs

**Checkpoint**: Anti-thrashing tests pass, SafeMode works

---

## Phase 6: User Story 4 - Audit Completeness (Priority: P2)

**Goal**: Every proposal/apply/rollback is logged with no silent drops

**Independent Test**: Run 1000 cycles; verify audit log contains exactly 1000 of each event type

### Tests for US4 (Write FIRST, ensure they FAIL)

- [ ] T054 [P] [US4] Test: Audit queue enqueues events with correlation IDs — tests/test_adaptive_engine.rs
- [ ] T055 [P] [US4] Test: Queue-full triggers SafeMode, never silent drop — tests/test_adaptive_engine.rs
- [ ] T056 [P] [US4] Test: 1000 proposals → 1000 audit events (no drops) — tests/test_adaptive_engine.rs

### Implementation for US4

- [ ] T057 [P] [US4] Create AuditEvent struct and EventType enum in adaptive_engine/audit.rs
- [ ] T058 [US4] Create AuditQueue with crossbeam::ArrayQueue in adaptive_engine/audit.rs
- [ ] T059 [US4] Implement EnqueueResult (Ok, HighWaterMark, Full) in adaptive_engine/audit.rs
- [ ] T060 [US4] Implement queue-full → SafeMode trigger in adaptive_engine/audit.rs
- [ ] T061 [US4] Integrate audit emission into SafeExecutor apply/rollback paths

**Checkpoint**: Audit tests pass, no silent drops

---

## Phase 7: Engine Orchestration

**Purpose**: Wire all components together into AdaptiveEngine

### Tests for Orchestration

- [ ] T062 [P] Test: AdaptiveEngine::new() initializes all components — tests/test_adaptive_engine.rs
- [ ] T063 [P] Test: Full cycle: observe → proposal → apply → audit — tests/test_adaptive_engine.rs
- [ ] T064 Integration test: Convergence under drift with no constraint violations — tests/test_adaptive_engine.rs

### Implementation

- [ ] T065 Create AdaptiveEngineConfig struct in adaptive_engine/mod.rs
- [ ] T066 Create AdaptiveEngine struct (orchestrator) in adaptive_engine/mod.rs
- [ ] T067 Implement AdaptiveEngine::new() wiring all components
- [ ] T068 Implement AdaptiveEngine::observe() delegating to proposer
- [ ] T069 Implement AdaptiveEngine::apply() delegating to executor with audit
- [ ] T070 Implement AdaptiveEngine::rollback() with audit
- [ ] T071 Implement AdaptiveEngine::set_baseline()
- [ ] T072 Implement AdaptiveEngine::current() returning snapshot

**Checkpoint**: Full engine works end-to-end

---

## Phase 8: Benchmarks & Performance

**Purpose**: Verify timing contracts

- [ ] T073 Create benchmark harness in benches/adaptive_engine_latency.rs
- [ ] T074 [P] Benchmark: T2_decision_us (target ≤1,000 µs p99)
- [ ] T075 [P] Benchmark: T1_apply_us (target ≤100 µs p99)
- [ ] T076 Add CI check to fail if timing budgets exceeded

**Checkpoint**: Timing contracts verified

---

## Phase 9: Polish & Documentation

**Purpose**: Finalize and document

- [ ] T077 [P] Create homeostasis.rs stub (schema only, impl deferred)
- [ ] T078 [P] Update crates/core/src/lib.rs with public API exports
- [ ] T079 [P] Add module-level rustdoc to adaptive_engine/mod.rs
- [ ] T080 Run quickstart.md validation to verify usage examples
- [ ] T081 Update CHANGELOG.md with 005-adaptive-engine entry

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) → Phase 2 (Foundational) → [Phases 3-6 in priority order]
                                          → Phase 7 (Orchestration)
                                          → Phase 8 (Benchmarks)
                                          → Phase 9 (Polish)
```

### User Story Dependencies

| Story | Depends On | Can Parallel With |
|:---|:---|:---|
| US1 (Adaptation) | Phase 2 | — |
| US2 (Safety) | Phase 2 | US1 models |
| US3 (Anti-Thrashing) | US2 (SafetyExecutor) | US4 |
| US4 (Audit) | Phase 2 | US3 |
| Orchestration | US1, US2, US3, US4 | — |

### Within Each User Story

1. Tests FIRST (verify they FAIL)
2. Core types (structs, enums)
3. Logic implementations
4. Integration with other components
5. Verify tests PASS

---

## Parallel Opportunities

### Phase 2 (Foundational)
```
Parallel: T006, T007, T010, T011, T012
```

### US1 (Adaptation)
```
Parallel: T014, T015, T016, T017 (tests)
Parallel: T018, T019 (TelemetryDigest, RingBuffer)
```

### US2 (Safety)
```
Parallel: T030, T031, T032, T033, T034 (tests)
Parallel: T035, T036 (Violation, Guardrails)
```

### US3 (Anti-Thrashing)
```
Parallel: T043, T044, T045, T046 (tests)
```

### US4 (Audit)
```
Parallel: T054, T055, T056 (tests)
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: US1 (Adaptation) — **STOP and TEST**
4. Complete Phase 4: US2 (Safety) — **STOP and TEST**
5. **MVP Complete**: Basic safe adaptation works

### Full Implementation

6. Complete Phase 5: US3 (Anti-Thrashing)
7. Complete Phase 6: US4 (Audit)
8. Complete Phase 7: Orchestration
9. Complete Phase 8: Benchmarks
10. Complete Phase 9: Polish

---

## Summary

| Phase | Tasks | Tests | Parallel |
|:---|:---|:---|:---|
| Setup | 4 | — | 2 |
| Foundational | 9 | 4 | 5 |
| US1 (P1) | 16 | 4 | 6 |
| US2 (P1) | 13 | 5 | 7 |
| US3 (P2) | 11 | 4 | 5 |
| US4 (P2) | 8 | 3 | 3 |
| Orchestration | 11 | 3 | 2 |
| Benchmarks | 4 | — | 2 |
| Polish | 5 | — | 3 |
| **Total** | **81** | **23** | **35** |

**MVP Scope**: Phases 1-4 (42 tasks)  
**Full Scope**: All phases (81 tasks)
