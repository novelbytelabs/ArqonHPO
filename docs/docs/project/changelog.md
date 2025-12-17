# Changelog

All notable changes to ArqonHPO will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
- Updated `docs/docs/cookbook/sim_tuning.md` with PCR algorithm explanation
- Updated `docs/docs/cookbook/ml_tuning.md` with Scott's Rule bandwidth details

### Tests

- 36 Rust unit tests for all PCR components
- 3 Python integration tests (test_integration, test_us1, test_us2)
