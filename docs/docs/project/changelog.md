# Changelog

All notable changes to ArqonHPO will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.0] - 2026-01-09

### Added
- **Safety Executor** — Centralized config updates with guardrails
- **Guardrails presets** — Conservative, Balanced, Aggressive
- **Rollback Policy** — Automatic revert on regressions
- **Dashboard API** — REST endpoints for monitoring
- **TUI** — Terminal interface for real-time monitoring
- **TPE Strategy** — Tree-structured Parzen Estimator
- **ArqonProbe.sample_range()** — Batch LDS sampling
- **ask_one()** — Single candidate for online mode

### Changed
- `probe_budget` config → `probe_ratio` (breaking)
- Default batch size 4 → auto-scaled by budget
- Improved SPSA gradient estimation with trimmed mean

### Fixed
- Memory leak in TPE kernel caching
- Race condition in audit queue flush

### Documentation
- Comprehensive reference docs (40+ pages)
- TUI/Dashboard screenshots
- Integration guides (K8s, FastAPI, Ray)

---

## [0.2.0] - 2025-09-15

### Added
- **PCR Algorithm** — Probe-Classify-Refine pipeline
- **Python bindings** — `ArqonSolver`, `ArqonProbe`
- **ResidualDecayClassifier** — Landscape classification
- **Multi-Start Nelder-Mead** — Multimodal optimization
- **Scott's Rule TPE Bandwidth** — Adaptive kernel bandwidth
- CLI commands: `ask`, `tell`, `interactive`, `validate`

### Changed
- Renamed CLI: `arqon` → `arqonhpo`
- Renamed package: `arqon` → `arqonhpo`

### Documentation
- Initial MkDocs site
- Quickstart guide
- Python/Rust reference

---

## [0.1.0] - 2025-06-01

### Added
- Initial release
- **Nelder-Mead** simplex optimizer
- Basic CLI: `run` command
- Deterministic RNG with ChaCha8
- State persistence (JSON)
- Prometheus metrics endpoint

---

## Upgrade Notes

### 0.2 → 0.3
- Rename `probe_budget` to `probe_ratio` in config
- Update imports: `from arqonhpo import ArqonSolver`
- Re-export artifacts (schema change)

See [Migration Guide](../documentation/migration.md) for details.

### 0.1 → 0.2
- Complete config rewrite — recreate configs
- Package renamed — update pip install

---

## Links

- [GitHub Releases](https://github.com/novelbytelabs/ArqonHPO/releases)
- [PyPI](https://pypi.org/project/arqonhpo/)
- [Roadmap](roadmap.md)
