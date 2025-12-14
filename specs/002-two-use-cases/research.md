# Research & Decisions: ArqonHPO v1

**Branch**: `002-two-use-cases`
**Date**: 2025-12-13

## Summary
This document resolves the open questions from `spec.md` (R1, R2) and documents the rationale for key technical choices, including Production Readiness (CI/CD, Docs).

## Resolved Spec Questions

### R1: Exact Target Thresholds
**Question**: What are the exact time-to-target thresholds for the benchmark suite?
**Decision**: **Defer to Evidence.**
**Rationale**: We cannot set meaningful improvement targets (e.g., "reach error 0.01 in 5s") without baseline measurements of the specific objective functions on the canonical hardware (`helios-gpu-118`).
**Action**: The first task in the `benchmark` phase will be "Establish Baselines" using pure TPE. The acceptance criteria for ArqonHPO will then be relative (e.g., ">15% faster median time-to-target") rather than absolute.

### R2: Structured vs. Chaotic Thresholds
**Question**: What is the numeric threshold for the variance classifier?
**Decision**: **Defer to Calibration.**
**Rationale**: The `variance_score` distribution depends heavily on the specific sampling method (Sobol/Halton) and the objective scale. Hardcoding a threshold now is guessing.
**Action**: We will expose this as a configurable `classification_threshold` in `SolverConfig` with a safe initial default, and tune it against the "Sim tuning" benchmark during implementation.

## Technical Decisions

### D1: Rust Workspace Structure
**Decision**: A standard Cargo workspace with explicit separation of concerns.
**Rationale**:
- `crates/core`: The pure logic library (no I/O, no FFI). Easy to test.
- `crates/cli`: Thin wrapper for binary distribution.
- `crates/ffi`: C-ABI export layer (future proofing).
- `bindings/python`: PyO3 integration.
**Alternatives**: Monorepo with top-level crate. rejected to enforce separation of concerns as per Constitution.

### D2: Random Number Generation
**Decision**: Use `rand_chacha` (ChaCha20) for the `Rng` implementation.
**Rationale**: We need cryptographically strong (statistic-passing) pseudo-randomness that is strictly reproducible across platforms including WASM. Standard `rand::StdRng` is not guaranteed to be portable across major versions; `rand_chacha` is stable.

### D3: Artifact Serialization
**Decision**: `serde_json` with a tagged enum format.
**Rationale**: JSON is the constitutional requirement. Tagged enums allow readable, type-safe differentiation between `Probe` and `Refine` steps in the trace.

### D4: Python Bridge
**Decision**: `PyO3` + `maturin`.
**Rationale**: `maturin` is the standard for building/publishing Rust-Python mixed projects. It handles the complexity of wheels and abi3 compatibility better than setuptools.

### D5: Production Readiness ("The Gamut")
**Decision**: Comprehensive 4-Layer Documentation Strategy.
**Rationale**: "Code is not the product." We need to serve four distinct audiences:
1.  **Users**: Narrative Guides + Cookbook (MkDocs).
2.  **Devs**: API Reference (Rustdoc/Mkdocstrings).
3.  **Ops**: Manpages/CLI Help (`clap`).
4.  **Architects**: ADRs (Architecture Decision Records) stored in-repo.
**Tooling**:
- `mkdocs-material`: The gold standard for microsites.
- `mike`: For versioned docs (v1.0, v1.1...).
- `mdbook`: For "The Book" (Rust conceptual guide).

---

## RPZL Algorithm Decisions (Session 2025-12-14)

### D6: Classification Algorithm
**Decision**: Residual Decay Analysis with α < 0.5 threshold.
**Rationale**: Coefficient of Variation (CV) incorrectly classifies high-range smooth functions as chaotic. Residual decay directly measures smoothness by analyzing how errors decrease across iterative refinement passes.
**Alternative Rejected**: CV-based classification (too sensitive to objective value ranges).

### D7: TPE Bandwidth Selection
**Decision**: Scott's Rule (σ = 1.06 × stddev × n^(-1/5)).
**Rationale**: Standard KDE bandwidth estimation that adapts to sample distribution. Fixed 10% range is too coarse for narrow parameter ranges and too fine for wide ranges.
**Alternative Rejected**: Fixed percentage (doesn't adapt to data), Silverman's Rule (more complex, marginal benefit).

### D8: Nelder-Mead Operations
**Decision**: Implement all 5 operations (Reflection, Expansion, Outside Contraction, Inside Contraction, Shrink).
**Rationale**: Current implementation only has reflection, which causes the algorithm to stall on any non-trivial optimization. Full NM is required for proper convergence.
**Alternative Rejected**: Reflection-only (incomplete algorithm, cannot converge).

### D9: Probe Sampling Strategy
**Decision**: Prime-Index Sampling (sample at prime ratios p_k/N).
**Rationale**: RPZL methodology uses prime-index probing for multi-scale structure detection without aliasing. Deterministic and simple to implement.
**Alternative Rejected**: Uniform random (poor multi-scale coverage), Sobol (requires external library).

### D10: Probe-to-Refiner Seeding
**Decision**: Initialize NM simplex from top-k best probe points, with perturbations for remaining vertices.
**Rationale**: The RPZL "secret sauce" - using probe information to seed the refiner dramatically improves convergence by starting from known-good regions.
**Alternative Rejected**: Random init (ignores probe signal), centroid-only (insufficient diversity).
