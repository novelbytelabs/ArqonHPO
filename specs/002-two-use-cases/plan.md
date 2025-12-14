# Implementation Plan: ArqonHPO v1 (Two Use Cases)

**Branch**: `002-two-use-cases` | **Date**: 2025-12-13 | **Spec**: [v1.0](../spec.md)
**Status**: APPROVED

## Summary
Implement the ArqonHPO probe-gated optimization engine as a **Rust Core** library with **PyO3 Python bindings**. The system will strictly follow the `Probe -> Classify -> Select -> Refine` pipeline to deliver deterministic, bounded-overhead performance for "Sim Tuning" and "Sklearn Tuning" use cases.

## Technical Context

**Language**: Rust 1.75+ (Core), Python 3.10+ (SDK)
**Primary Dependencies**:
- Rust: `serde` (serialization), `rand_chacha` (determinism), `pyo3` (bindings), `thiserror` (error handling).
- Python: `numpy` (numeric types).
**Testing**:
- Rust: `cargo test` (Unit/Prop).
- Python: `pytest` (End-to-End).
**Target Platform**: Linux (x86_64), macOS (universal), Windows (x86_64).
**Performance Goals**: < 100µs optimizer overhead per step (Rust side).
**Constraints**: Strict O(1) memory/compute per eval.

## Constitution Check
*GATE: Passed*

- [x] **Warm-start enforcement**: The `Solver` struct state machine will enforce strictly typed transitions (Probe -> Classify -> Refine).
- [x] **No bypass**: The `step()` method will match on the internal state; no user-facing method allows skipping phases.
- [x] **Purity**: The Core crate will be `#[no_std]` compatible (conceptually) - pure logic, no I/O. I/O happens only in the CLI/Bindings layer.
- [x] **Reproducibility**: `SolverConfig` captures all inputs. `RunArtifact` captures the fingerprint.
- [x] **Canonical Env**: CI pipeline will use `helios-gpu-118` equivalent container.

## Project Structure

### Source Code (Cargo Workspace)
```text
Cargo.toml (workspace)
crates/
├── core/                  # arqonhpo-core (Library)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs      # Typed contracts
│   │   ├── machine.rs     # State machine logic
│   │   ├── probe.rs       # Deterministic samplers (Sobol/Grid)
│   │   ├── strategies/    # Nelder-Mead, TPE
│   │   └── artifact.rs    # JSON schema structs
│   └── Cargo.toml
├── cli/                   # arqonhpo-cli (Binary)
│   ├── src/main.rs
│   └── Cargo.toml
└── ffi/                   # arqonhpo-ffi (C-ABI)
    └── Cargo.toml

bindings/
└── python/                # arqonhpo (Python Package)
    ├── src/lib.rs         # PyO3 bindings
    ├── python/
    │   └── arqonhpo/      # Type stubs (.pyi) and helpers
    ├── pyproject.toml     # Maturin config
    └── tests/             # Pytest suite
```

## Data Model & Contracts

### 1. SolverConfig (The Input)
The single source of truth for a run.
- `budget`: u64
- `seed`: u64
- `probe_ratio`: f64 (Default 0.2)
- `bounds`: Map<String, Domain>
- `defaults`: StrategyConfig

### 2. RunArtifact (The Output)
The schema-versioned evidence track.
- `meta`: Version, Timestamp, Fingerprint
- `config`: SolverConfig
- `trace`: List<StepEvent> (Probe | Classify | Refine)
- `result`: BestParam, BestValue, TotalDuration

## Implementation Phases

### Phase 1: The Core Foundation
- [ ] Setup Cargo workspace and crates.
- [ ] Implement `SolverConfig` and `Domain` types with Serde.
- [ ] Implement `RunArtifact` schema and serialization.
- [ ] Implement Deterministic RNG wrapper (`rand_chacha`).

### Phase 2: The Solver State Machine
- [ ] Implement `Probe` phase (Sobol sampler).
- [ ] Implement `Classify` phase (Variance calculator).
- [ ] Implement `State` transitions (Probe -> Classify -> Refine).

### Phase 3: Refinement Strategies (MVP)
- [ ] Implement `Nelder-Mead` (Structured).
- [ ] Implement `TPE` (Chaotic) - *Note: TPE is complex, we may start with Random for MVP alpha if TPE is too heavy, but spec says TPE.*

### Phase 4: Bindings & CLI
- [ ] Implement `ArqonSolver` PyO3 class.
- [ ] Implement CLI `run` command.

### Phase 5: Verification & Benchmarking
- [ ] Write integration tests in Python.
- [ ] Run benchmark suite.

### Phase 6: Top-Tier Documentation ("The Gamut")
**Strategy**: Implement the "Five Pillars" strategy directly in the repo.
- [ ] **[Foundation]** Init `mkdocs-material` site with `mike` versioning.
- [ ] **[Content - Tutorials]** Write "First Run" Quickstart.
- [ ] **[Content - Cookbook]** Implement "The Cookbook" with 10+ recipes (Ref: `docs-strategy.md`).
- [ ] **[Content - Reference]** Setup auto-gen pipelines:
  - Rust: `cargo doc` hosted at `/rust`
  - Python: `mkdocstrings` hosted at `/python`
  - CLI: `clap_mangen` hosted at `/cli`
- [ ] **[Content - Explanation]** Write ADR-001 (Core) and ADR-002 (Bridge).
- [ ] **[Community]** Create `README.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`.
- [ ] **[Ops]** Write `RELEASE_RUNBOOK.md` and `CI_DEBUG_RUNBOOK.md`.

### Phase 7: The "Million Dollar" Polish
**Strategy**: Integrate "Elite" tooling capability.
- [ ] **[Security]** Add `slsa-framework/slsa-github-generator` to Actions workflow.
- [ ] **[Security]** Add `anchore/sbom-action` for CycloneDX generation.
- [ ] **[Quality]** Create `fuzz/` directory and add `cargo-fuzz` target for `SolverConfig`.
- [ ] **[DX]** Replace `anyhow` with `miette` in `crates/cli` and `crates/core` for labeled errors.
- [ ] **[DX]** Register `contracts/config.json` with SchemaStore (PR to `SchemaStore/schemastore`).
- [ ] **[Ops]** Instrument `step()` loop with `tracing::span!` and `tracing-subscriber`.
- [ ] **[Release]** Configure `python-semantic-release` or standard `release-plz` for semantic versioning.

