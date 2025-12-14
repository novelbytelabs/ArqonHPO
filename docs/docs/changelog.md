# Changelog

All notable changes to ArqonHPO will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **RPZL Algorithm Implementation** - Complete production-ready optimization pipeline:
  - `ResidualDecayClassifier`: Automatic landscape classification using α estimation from residual decay curves. α > 0.5 → Structured (Nelder-Mead), α ≤ 0.5 → Chaotic (TPE)
  - `PrimeIndexProbe`: Multi-scale sampling using prime ratios via Sieve of Eratosthenes for better structure detection
  - `Solver::rpzl()`: Production constructor combining all RPZL components with Top-K probe seeding
  - `SeedingConfig`: Configurable Top-K seeding for Nelder-Mead simplex initialization

- **Full Nelder-Mead Implementation** - All 5 standard operations:
  - Reflection, Expansion, Outside Contraction, Inside Contraction, Shrink
  - `NMCoefficients` struct with standard values (α=1.0, γ=2.0, ρ=0.5, σ=0.5)
  - Convergence detection based on simplex diameter
  - `with_seed_points()` constructor for probe-based initialization

- **Scott's Rule TPE Bandwidth** - Adaptive kernel bandwidth:
  - `BandwidthRule` enum: Scott, Silverman, Fixed
  - Scott's Rule: σ = 1.06 × stddev × n^(-1/5)
  - `TPE::with_bandwidth_rule()` constructor

### Changed

- `Solver::new()` now uses `SeedingConfig::default()` for probe-to-refiner seeding
- Replaced fixed 10% bandwidth in TPE with adaptive Scott's Rule

### Documentation

- Updated `docs/docs/reference/rust.md` with new API documentation
- Updated `docs/docs/cookbook/sim_tuning.md` with RPZL algorithm explanation
- Updated `docs/docs/cookbook/ml_tuning.md` with Scott's Rule bandwidth details

### Tests

- 36 Rust unit tests for all RPZL components
- 3 Python integration tests (test_integration, test_us1, test_us2)
