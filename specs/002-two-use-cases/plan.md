# Implementation Plan: ArqonHPO v1 (Two Use Cases)

**Branch**: `002-two-use-cases` | **Date**: 2025-12-13 | **Spec**: [v1.0](../spec.md)
**Status**: APPROVED

## Summary
Implement the ArqonHPO probe-gated optimization engine as a **Rust Core** library with **PyO3 Python bindings**. The system will strictly follow the `Probe -> Classify -> Select -> Refine` pipeline to deliver deterministic, bounded-overhead performance for "Sim Tuning" and "Sklearn Tuning" use cases.

## Technical Context

**Language**: Rust 1.75+ (Core), Python 3.10+ (SDK)
**Primary Dependencies**:
- Rust: `serde` (serialization), `rand_chacha` (determinism), `pyo3` (bindings), `miette` (rich errors), `tracing` (telemetry).
- Python: `numpy` (numeric types).
**Testing**:
- Rust: `cargo test` (Unit/Prop), `cargo-fuzz` (Stability).
- Python: `pytest` (End-to-End).
**Target Platform**: Linux (x86_64), macOS (universal), Windows (x86_64).
*Performance Goals**: < 100µs optimizer overhead.
**Constraints**: Strict O(1) memory/compute.

## Constitution Check
*GATE: Passed*

- [x] **Warm-start enforcement**: `Solver` state machine enforced.
- [x] **No bypass**: No `step()` bypass allowed.
- [x] **Purity**: `no_std` compatible core logic.
- [x] **Reproducibility**: Config & Artifact capture.
- [x] **Canonical Env**: CI uses `helios-gpu-118`.
- [x] **Production Readiness**: Plan includes SLSA, Fuzzing, DX, and Docs (NFR 7.1).

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
│   │   ├── probe.rs       # Deterministic samplers
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
    ├── src/lib.rs
    ├── python/
    │   └── arqonhpo/      # Type stubs (.pyi)
    ├── pyproject.toml     # Maturin config
    └── tests/
```

## Data Model & Contracts (See contracts/)
- `SolverConfig`: Input definition.
- `RunArtifact`: Output schema.

## Implementation Phases

### Phase 1: The Core Foundation
- [ ] Setup Cargo workspace and crates.
- [ ] Implement `SolverConfig`/`RunArtifact` with Serde.
- [ ] Implement Deterministic RNG (`rand_chacha`).

### Phase 2: The Solver State Machine
- [ ] Implement `Probe` (Sobol) and `Classify` (Variance).
- [ ] Implement `State` transitions.

### Phase 3: Refinement Strategies (MVP)
- [ ] Implement `Nelder-Mead`.
- [ ] Implement `TPE` (Chaotic).

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
