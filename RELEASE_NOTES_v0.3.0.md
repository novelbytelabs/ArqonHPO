# v0.3.0 - PCR Pipeline & Adaptive TPE

**Release Date**: 2026-01-09

## 🚀 Highlights

This release completes the production-grade **Probe-Classify-Refine (PCR)** pipeline and ships a full **Nelder-Mead** implementation with adaptive **Scott's Rule TPE** bandwidth selection.

### 🧪 PCR Pipeline

- **ResidualDecayClassifier** for automatic landscape classification (Structured vs Chaotic)
- **PrimeIndexProbe** multi-scale sampling using prime ratios
- **SeedingConfig** for Top-K probe seeding into Nelder-Mead
- `Solver::pcr()` constructor for the full pipeline

### 📈 Optimizers

- **Full Nelder-Mead** with all 5 operations and convergence detection
- **Scott's Rule** adaptive bandwidth for TPE (`BandwidthRule::Scott`)

### 📚 Documentation & Tests

- Updated Rust API docs and tuning cookbooks
- 36 Rust unit tests + 3 Python integration tests for PCR components

## ⬆️ Upgrade Notes

No breaking changes expected. This release refines defaults and adds new constructors for the PCR workflow.

---

**Full Changelog**: https://github.com/novelbytelabs/ArqonHPO/compare/v0.2.0...v0.3.0
