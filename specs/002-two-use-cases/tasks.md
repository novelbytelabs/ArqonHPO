# Tasks: RPZL Algorithm Production Implementation

**Input**: Design documents from `/specs/002-two-use-cases/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅

**Tests**: REQUIRED per constitution - TDD for solver logic and sampling math.

**Organization**: Tasks grouped by implementation phase, mapped to User Stories (US1: Sim Tuning, US2: ML Tuning).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1 = Fast Simulation Tuning, US2 = Sklearn ML Tuning
- Include exact file paths in descriptions

---

## Phase 1: Setup (Test Infrastructure)

**Purpose**: Create test infrastructure before implementation (TDD)

- [x] T001 Create test module structure at `crates/core/src/tests/mod.rs`
- [x] T002 [P] Create test file for classifier at `crates/core/src/tests/test_classify.rs`
- [x] T003 [P] Create test file for TPE at `crates/core/src/tests/test_tpe.rs`
- [x] T004 [P] Create test file for Nelder-Mead at `crates/core/src/tests/test_nelder_mead.rs`
- [x] T005 [P] Create test file for probe at `crates/core/src/tests/test_probe.rs`
- [x] T006 Add test module declaration to `crates/core/src/lib.rs`

**Checkpoint**: Test infrastructure ready, all tests should compile and fail.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core algorithm implementations that MUST be complete before user story validation

**⚠️ CRITICAL**: User story integration tests cannot pass until this phase is complete

### 2.1 Residual Decay Classifier

- [x] T007 [P] Write failing test for α estimation in `crates/core/src/tests/test_classify.rs`
- [x] T008 [P] Write failing test for Sphere→Structured classification in `crates/core/src/tests/test_classify.rs`
- [x] T009 [P] Write failing test for Rastrigin→Chaotic classification in `crates/core/src/tests/test_classify.rs`
- [x] T010 Add `ResidualDecayClassifier` struct to `crates/core/src/classify.rs`
- [x] T011 Implement α estimation from E_k decay curve in `crates/core/src/classify.rs`
- [x] T012 Implement `Classify` trait for `ResidualDecayClassifier` with α < 0.5 threshold in `crates/core/src/classify.rs`
- [x] T013 Update `Solver` to use `ResidualDecayClassifier` as default in `crates/core/src/machine.rs`

### 2.2 Scott's Rule TPE Bandwidth

- [ ] T014 [P] Write failing test for Scott's Rule σ calculation in `crates/core/src/tests/test_tpe.rs`
- [ ] T015 [P] Write failing test for bandwidth adaptation across dimensions in `crates/core/src/tests/test_tpe.rs`
- [ ] T016 Add `scotts_bandwidth(values: &[f64]) -> f64` function to `crates/core/src/strategies/tpe.rs`
- [ ] T017 Replace fixed `sigma = range * 0.1` with `scotts_bandwidth()` in `crates/core/src/strategies/tpe.rs`
- [ ] T018 Add minimum bandwidth clamp (1e-6) to prevent degenerate kernels in `crates/core/src/strategies/tpe.rs`

### 2.3 Complete Nelder-Mead Operations

- [ ] T019 [P] Write failing test for Expansion operation in `crates/core/src/tests/test_nelder_mead.rs`
- [ ] T020 [P] Write failing test for Outside Contraction in `crates/core/src/tests/test_nelder_mead.rs`
- [ ] T021 [P] Write failing test for Inside Contraction in `crates/core/src/tests/test_nelder_mead.rs`
- [ ] T022 [P] Write failing test for Shrink operation in `crates/core/src/tests/test_nelder_mead.rs`
- [ ] T023 [P] Write failing test for simplex convergence on Sphere in `crates/core/src/tests/test_nelder_mead.rs`
- [ ] T024 Implement Expansion handler in `crates/core/src/strategies/nelder_mead.rs` (γ = 2.0)
- [ ] T025 Implement Outside Contraction handler in `crates/core/src/strategies/nelder_mead.rs` (ρ = 0.5)
- [ ] T026 Implement Inside Contraction handler in `crates/core/src/strategies/nelder_mead.rs` (ρ = 0.5)
- [ ] T027 Implement Shrink handler in `crates/core/src/strategies/nelder_mead.rs` (σ = 0.5)
- [ ] T028 Add convergence detection (simplex diameter < ε) in `crates/core/src/strategies/nelder_mead.rs`

### 2.4 Prime-Index Probe

- [ ] T029 [P] Write failing test for prime sequence generation in `crates/core/src/tests/test_probe.rs`
- [ ] T030 [P] Write failing test for deterministic sampling in `crates/core/src/tests/test_probe.rs`
- [ ] T031 Add `PrimeIndexProbe` struct to `crates/core/src/probe.rs`
- [ ] T032 Implement Sieve of Eratosthenes for prime generation (up to 1000) in `crates/core/src/probe.rs`
- [ ] T033 Implement `Probe` trait for `PrimeIndexProbe` with prime ratio sampling in `crates/core/src/probe.rs`
- [ ] T034 Update `Solver` to use `PrimeIndexProbe` as default in `crates/core/src/machine.rs`

### 2.5 Probe-to-Refiner Seeding

- [ ] T035 [P] Write failing test for NM seeding from probe points in `crates/core/src/tests/test_nelder_mead.rs`
- [ ] T036 Add `with_seed_points(dim: usize, seeds: Vec<(f64, Vec<f64>)>) -> Self` to `crates/core/src/strategies/nelder_mead.rs`
- [ ] T037 Modify `NMState::Init` to use seed points as first k simplex vertices in `crates/core/src/strategies/nelder_mead.rs`
- [ ] T038 Add perturbation generation for remaining (N+1-k) vertices in `crates/core/src/strategies/nelder_mead.rs`
- [ ] T039 Update `Solver` to pass top-k probe results to NM in `crates/core/src/machine.rs`

**Checkpoint**: `cargo test --package arqonhpo-core` should pass all unit tests.

---

## Phase 3: User Story 1 - Fast Simulation Tuning (Priority: P1) 🎯 MVP

**Goal**: Reach target threshold faster than Optuna-TPE on structured objectives

**Independent Test**: Run solver on Sphere function → should classify as Structured, converge faster than baseline

### Tests for User Story 1

- [ ] T040 [P] [US1] Write integration test for Sphere optimization in `bindings/python/tests/test_us1_sphere.py`
- [ ] T041 [P] [US1] Write integration test for Rosenbrock optimization in `bindings/python/tests/test_us1_rosenbrock.py`
- [ ] T042 [US1] Write time-to-target comparison test vs baseline in `bindings/python/tests/test_us1_benchmark.py`

### Implementation for User Story 1

- [ ] T043 [US1] Add structured objective fixtures (Sphere, Rosenbrock) to `bindings/python/tests/fixtures/structured.py`
- [ ] T044 [US1] Run end-to-end test: Sphere classification → Structured → NM refinement in Python
- [ ] T045 [US1] Verify probe seeding improves convergence vs random init
- [ ] T046 [US1] Add logging for classification rationale in artifact output

**Checkpoint**: US1 complete - Structured objectives reach target faster than baseline.

---

## Phase 4: User Story 2 - Sklearn ML Tuning (Priority: P2)

**Goal**: Competitive time-to-target with bounded optimizer overhead on chaotic objectives

**Independent Test**: Run solver on Rastrigin function → should classify as Chaotic, use TPE with adapted bandwidth

### Tests for User Story 2

- [ ] T047 [P] [US2] Write integration test for Rastrigin optimization in `bindings/python/tests/test_us2_rastrigin.py`
- [ ] T048 [P] [US2] Write integration test for sklearn SGDClassifier tuning in `bindings/python/tests/test_us2_sklearn.py`

### Implementation for User Story 2

- [ ] T049 [US2] Add chaotic objective fixtures (Rastrigin, Ackley) to `bindings/python/tests/fixtures/chaotic.py`
- [ ] T050 [US2] Run end-to-end test: Rastrigin classification → Chaotic → TPE refinement in Python
- [ ] T051 [US2] Verify Scott's Rule bandwidth adapts correctly per dimension
- [ ] T052 [US2] Verify optimizer overhead < 100µs per evaluation

**Checkpoint**: US2 complete - Chaotic objectives handled with competitive performance.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, benchmarks, and final validation

- [ ] T053 [P] Update `docs/docs/reference/rust.md` with new classifier/probe API
- [ ] T054 [P] Update `docs/docs/cookbook/sim_tuning.md` with RPZL algorithm explanation
- [ ] T055 [P] Update `docs/docs/cookbook/ml_tuning.md` with TPE bandwidth details
- [ ] T056 Run full benchmark suite: `python benchmarks/run_benchmarks.py`
- [ ] T057 Generate benchmark report comparing ArqonHPO vs Optuna-TPE
- [ ] T058 Run `specs/002-two-use-cases/quickstart.md` validation
- [ ] T059 Update `CHANGELOG.md` with RPZL algorithm improvements

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 completion
- **Phase 4 (US2)**: Depends on Phase 2 completion (can parallel with US1)
- **Phase 5 (Polish)**: Depends on US1 + US2 completion

### Within Phase 2 (Foundational)

```text
T007-T009 (tests) → T010-T013 (classifier impl)
T014-T015 (tests) → T016-T018 (TPE impl)
T019-T023 (tests) → T024-T028 (NM impl)
T029-T030 (tests) → T031-T034 (probe impl)
T035 (test) → T036-T039 (seeding impl)
```

### Parallel Opportunities

**Phase 1**:
- T002, T003, T004, T005 can run in parallel

**Phase 2**:
- All test tasks (T007-T009, T014-T015, T019-T023, T029-T030, T035) can run in parallel
- Classifier (T010-T013) and TPE (T016-T018) and Probe (T031-T034) can run in parallel after their tests

**Phase 3+**:
- US1 and US2 can run in parallel once Phase 2 is complete

---

## Parallel Example: Phase 2 Tests

```bash
# Launch all Phase 2 tests together:
cargo test --package arqonhpo-core test_classify -- --nocapture
cargo test --package arqonhpo-core test_tpe -- --nocapture
cargo test --package arqonhpo-core test_nelder_mead -- --nocapture
cargo test --package arqonhpo-core test_probe -- --nocapture
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: All foundational tasks (T007-T039)
3. Complete Phase 3: US1 (T040-T046)
4. **VALIDATE**: `cargo test && pytest bindings/python/tests/test_us1*.py`
5. Demo MVP: Structured objectives work correctly

### Verification Commands

```bash
# Rust unit tests
cd /home/irbsurfer/Projects/arqon/ArqonHPO
cargo test --package arqonhpo-core

# Python integration tests
conda run -n helios-gpu-118 pytest bindings/python/tests/ -v

# Full benchmark
conda run -n helios-gpu-118 python benchmarks/run_benchmarks.py
```

---

## Summary

| Phase | Tasks | Parallel Tasks |
|-------|-------|----------------|
| Setup | T001-T006 | 4 |
| Foundational | T007-T039 | 15 |
| US1 (MVP) | T040-T046 | 2 |
| US2 | T047-T052 | 2 |
| Polish | T053-T059 | 3 |
| **Total** | **59** | **26** |
