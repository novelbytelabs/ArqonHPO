---
description: "Task list for ArqonHPO v1 implementation"
---

# Tasks: ArqonHPO v1 "Two Use Cases"

**Input**: Specs from `specs/002-two-use-cases/`
**Prerequisites**: plan.md, spec.md
**Organization**: Phases 1-2 (Foundation) -> Phases 3-4 (User Stories) -> Phase 5 (Polish/Docs)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize Rust workspace and Python environment.

- [x] T001 Create Cargo workspace with `core`, `cli`, `ffi`, `bindings/python` crates <!-- id: T001 -->
- [x] T002 Configure `Cargo.toml` workspace dependencies (serde, rand, pyo3) <!-- id: T002 -->
- [x] T003 [P] Setup Python environment (uv/poetry) and `pyproject.toml` (maturin) <!-- id: T003 -->
- [x] T004 [P] Configure development tools (Justfile, pre-commit, clippy, rustfmt) <!-- id: T004 -->

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core state machine and contracts that US1/US2 depend on.
**⚠️ CRITICAL**: Must be complete before strategies.

- [x] T005 Implement `SolverConfig` and `RunArtifact` structs with Serde in `crates/core` <!-- id: T005 -->
- [x] T006 Implement Deterministic RNG wrapper (`rand_chacha`) with seed management <!-- id: T006 -->
- [x] T007 Implement the `Probe` trait and Sobol sequence sampler <!-- id: T007 -->
- [x] T008 Implement the `Classify` trait and Variance-based classifier <!-- id: T008 -->
- [x] T009 Implement the `Solver` state machine (Probe -> Classify -> Select -> Refine) <!-- id: T009 -->
- [x] T010 Create `ArqonSolver` PyO3 wrapper class in `bindings/python` <!-- id: T010 -->
- [x] T011 Implement integration test harness in Python <!-- id: T011 -->

**Checkpoint**: Core engine compels the state machine loop; Python can reference it.

---

## Phase 3: User Story 1 - Fast Simulation Tuning (P0)

**Goal**: Nelder-Mead strategy for expensive, smooth simulations.

### Tests for US1
- [x] T012 [P] [US1] Create synthetic "smooth expensive" benchmark function (sleep-injected) <!-- id: T012 -->
- [x] T013 [P] [US1] Write integration test expecting Nelder-Mead selection for smooth surface <!-- id: T013 -->

### Implementation for US1
- [x] T014 [US1] Implement `Strategy` trait in `crates/core` <!-- id: T014 -->
- [x] T015 [US1] Implement `NelderMead` strategy logic (simplex) <!-- id: T015 -->
- [x] T016 [US1] Wire `NelderMead` into `Solver` selection logic (Structured mode) <!-- id: T016 -->
- [x] T017 [US1] Expose strategy configuration in `SolverConfig` (Rust & Python) <!-- id: T017 -->
- [x] T018 [US1] Verify determinism and artifacts for Sim Tuning run <!-- id: T018 -->

**Checkpoint**: Sim Engineers can use ArqonHPO for smooth functions.

---

## Phase 4: User Story 2 - Sklearn ML Tuning (P0)

**Goal**: TPE strategy for rugged/ML landscapes.

### Tests for US2
- [ ] T019 [P] [US2] Create synthetic "rugged" benchmark function <!-- id: T019 -->
- [ ] T020 [P] [US2] Write integration test expecting TPE selection for rugged surface <!-- id: T020 -->

### Implementation for US2
- [ ] T021 [US2] Implement `TPE` (Tree-structured Parzen Estimator) strategy logic <!-- id: T021 -->
- [ ] T022 [US2] Wire `TPE` into `Solver` selection logic (Chaotic mode) <!-- id: T022 -->
- [ ] T023 [US2] Ensure `PYTHONNOUSERSITE` compatibility for benchmark runs <!-- id: T023 -->
- [ ] T024 [US2] Run comparison benchmark vs Optuna (Time-to-target) <!-- id: T024 -->

**Checkpoint**: ML Engineers can use ArqonHPO for model tuning.

---

## Phase 5: Production Readiness & Documentation ("The Gamut")

**Purpose**: "Million Dollar" Polish NFRs and Documentation.

### Documentation (The Gamut)
- [ ] T025 [P] Init `mkdocs-material` site with `mike` and deployment workflow <!-- id: T025 -->
- [ ] T026 [P] Write "First Run" Quickstart Tutorial <!-- id: T026 -->
- [ ] T027 [P] Implement Cookbook with Sim and ML recipes <!-- id: T027 -->
- [ ] T028 [P] Setup Reference docs pipeline (Rustdoc, Mkdocstrings, Manpages) <!-- id: T028 -->
- [ ] T029 Write ADR-001 (Core Arch) and ADR-002 (Python Bridge) <!-- id: T029 -->
- [ ] T030 Add Community files (`CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`) <!-- id: T030 -->
- [ ] T031 Write OPS Runbooks (`RELEASE_RUNBOOK.md`, `CI_DEBUG_RUNBOOK.md`) <!-- id: T031 -->

### "Million Dollar" NFRs
- [ ] T032 [Security] Configure SLSA Level 3 generator in GitHub Actions <!-- id: T032 -->
- [ ] T033 [Security] Configure SBOM (CycloneDX) generation <!-- id: T033 -->
- [ ] T034 [Quality] Setup continuous fuzzing (`cargo-fuzz`) for `SolverConfig` <!-- id: T034 -->
- [ ] T035 [DX] Integrate `miette` for rich error reporting in Core/CLI <!-- id: T035 -->
- [ ] T036 [DX] Publish JSON Schema to SchemaStore (or prepare PR) <!-- id: T036 -->
- [ ] T037 [Ops] Instrument core with `tracing` spans <!-- id: T037 -->

---

## Dependencies & Execution Order

- Phase 2 (Foundation) BLOCKS Phase 3 & 4.
- Phase 3 & 4 can run in parallel.
- Phase 5 can start anytime but typically matches Phase 3/4 completion.
