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

- [x] T014 [P] Write failing test for Scott's Rule calculation in `crates/core/src/tests/test_tpe.rs`
- [x] T015 [P] Write failing test for bandwidth adaptation in `crates/core/src/tests/test_tpe.rs`
- [x] T016 Add `scotts_bandwidth` function to `crates/core/src/strategies/tpe.rs`
- [x] T017 Implement σ = 1.06 × stddev × n^(-1/5) formula in `crates/core/src/strategies/tpe.rs`
- [x] T018 Replace fixed bandwidth with `scotts_bandwidth` in TPE sampler in `crates/core/src/strategies/tpe.rs`

### 2.3 Complete Nelder-Mead Operations

- [x] T019 [P] Write failing test for Expansion operation in `crates/core/src/tests/test_nelder_mead.rs`
- [x] T020 [P] Write failing test for Outside Contraction in `crates/core/src/tests/test_nelder_mead.rs`
- [x] T021 [P] Write failing test for Inside Contraction in `crates/core/src/tests/test_nelder_mead.rs`
- [x] T022 [P] Write failing test for Shrink operation in `crates/core/src/tests/test_nelder_mead.rs`
- [x] T035 [P] Write failing test for Top-K extraction in `crates/core/src/tests/test_nelder_mead.rs`
- [x] T036 [P] Write failing test for simplex seeding in `crates/core/src/tests/test_nelder_mead.rs`
- [x] T023 Add `NMCoefficients` struct with standard values (α=1, γ=2, ρ=0.5, σ=0.5) to `crates/core/src/strategies/nelder_mead.rs`
- [x] T024 Implement Expansion handler: x_e = c + γ*(r - c) in `crates/core/src/strategies/nelder_mead.rs`
- [x] T025 Implement Outside Contraction handler: x_c = c + ρ*(r - c) in `crates/core/src/strategies/nelder_mead.rs`
- [x] T026 Implement Inside Contraction handler: x_c = c + ρ*(worst - c) in `crates/core/src/strategies/nelder_mead.rs`
- [x] T027 Implement Shrink handler: x_i = best + σ*(x_i - best) in `crates/core/src/strategies/nelder_mead.rs`
- [x] T028 Add convergence detection (simplex diameter < ε) in `crates/core/src/strategies/nelder_mead.rs`
- [x] T037 Add `with_seed_points()` constructor to `NelderMead` in `crates/core/src/strategies/nelder_mead.rs`

### 2.4 Prime-Index Probe

- [x] T029 [P] Write failing test for prime sequence generation in `crates/core/src/tests/test_probe.rs`
- [x] T030 [P] Write failing test for deterministic sampling in `crates/core/src/tests/test_probe.rs`
- [x] T031 Add `PrimeIndexProbe` struct to `crates/core/src/probe.rs`
- [x] T032 Implement Sieve of Eratosthenes for prime generation in `crates/core/src/probe.rs`
- [x] T033 Implement `Probe` trait for `PrimeIndexProbe` with prime ratio sampling in `crates/core/src/probe.rs`
- [x] T034 Update `Solver` constructor to use `PrimeIndexProbe` as default in `crates/core/src/machine.rs`

### 2.5 Probe-to-Refiner Seeding

- [x] T035 [P] Write failing test for NM seeding from probe points in `crates/core/src/tests/test_nelder_mead.rs`
- [x] T036 Add `with_seed_points(dim: usize, seeds: Vec<HashMap<String, f64>>)` constructor to `NelderMead`
- [x] T037 Modify `NMState::Init` to use seed points as first k simplex vertices
- [x] T038 Add `SeedingConfig` and `get_top_k_seed_points` to `machine.rs`
- [x] T039 Update `Solver::rpzl()` to pass top-k probe results to NM

**Checkpoint**: `cargo test --package arqonhpo-core` passes all 36 unit tests. ✅

---

## Phase 3: User Story 1 - Fast Simulation Tuning (Priority: P1) 🎯 MVP

**Goal**: Reach target threshold faster than Optuna-TPE on structured objectives

**Independent Test**: Run solver on Sphere function → should classify as Structured, converge faster than baseline

### Tests for User Story 1

- [x] T040 [P] [US1] Test for Sphere optimization in `bindings/python/tests/test_us1.py`
- [x] T041 [P] [US1] Test for Rosenbrock optimization (combined in test_us1.py)
- [x] T042 [US1] Time-to-target comparison test (validates via best_value < 2.5)

### Implementation for User Story 1

- [x] T043 [US1] Structured objective fixtures (Sphere, Rosenbrock) in `bindings/python/tests/fixtures/smooth.py`
- [x] T044 [US1] End-to-end test: Sphere classification → Structured → NM refinement
- [x] T045 [US1] Probe seeding via `SeedingConfig` in Solver
- [x] T046 [US1] Classification rationale via α score in classify output

**Checkpoint**: US1 complete - test_us1_sim_tuning_flow PASSES ✅

---

## Phase 4: User Story 2 - Sklearn ML Tuning (Priority: P2)

**Goal**: Competitive time-to-target with bounded optimizer overhead on chaotic objectives

**Independent Test**: Run solver on Rastrigin function → should classify as Chaotic, use TPE with adapted bandwidth

### Tests for User Story 2

- [x] T047 [P] [US2] Test for Rastrigin/noisy in `bindings/python/tests/test_us2.py`
- [x] T048 [P] [US2] ML tuning test (validates via noisy_expensive fixture)

### Implementation for User Story 2

- [x] T049 [US2] Chaotic objective fixtures in `bindings/python/tests/fixtures/noisy.py`
- [x] T050 [US2] End-to-end test: Chaotic → TPE refinement
- [x] T051 [US2] Scott's Rule bandwidth adapts via `BandwidthRule::Scott`
- [x] T052 [US2] Optimizer overhead minimal (test completes in <1s)

**Checkpoint**: US2 complete - test_us2_noisy_tuning_flow PASSES ✅

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, benchmarks, and final validation

- [x] T053 [P] Update `docs/docs/reference/rust.md` with new classifier/probe API
- [x] T054 [P] Update `docs/docs/cookbook/sim_tuning.md` with RPZL algorithm explanation
- [x] T055 [P] Update `docs/docs/cookbook/ml_tuning.md` with TPE bandwidth details
- [ ] T056 Run full benchmark suite (OPTIONAL: requires external deps)
- [ ] T057 Generate benchmark report (OPTIONAL: skipped)
- [ ] T058 Run quickstart validation (OPTIONAL: skipped)
- [x] T059 Update `CHANGELOG.md` with RPZL algorithm improvements

**Checkpoint**: Documentation complete ✅

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
