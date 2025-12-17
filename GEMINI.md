# ArqonHPO Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-14

## Active Technologies
- N/A (in-memory, artifacts to disk) (004-probe-upgrade)
- Rust 1.82+ (edition 2021) + `rand_chacha` (ChaCha8Rng), `rand` (Bernoulli), `smallvec` (ParamVec), `crossbeam` (lock-free audit queue) (005-adaptive-engine)
- N/A (in-memory only; audit events async-flushed to disk) (005-adaptive-engine)

- Rust 1.82+, Python 3.10+ (bindings) + `rand_chacha` (RNG), `serde` (serialization), `pyo3` (Python bindings) (002-two-use-cases)

## Project Structure

```text
src/
tests/
```

## Commands

cd src [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] pytest [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] ruff check .

## Code Style

Rust 1.82+, Python 3.10+ (bindings): Follow standard conventions

## Recent Changes
- 005-adaptive-engine: Added Rust 1.82+ (edition 2021) + `rand_chacha` (ChaCha8Rng), `rand` (Bernoulli), `smallvec` (ParamVec), `crossbeam` (lock-free audit queue)
- 004-probe-upgrade: Added Rust 1.82+, Python 3.10+ (bindings) + `rand_chacha` (RNG), `serde` (serialization), `pyo3` (Python bindings)

- 002-two-use-cases: Added Rust 1.82+, Python 3.10+ (bindings) + `rand_chacha` (RNG), `serde` (serialization), `pyo3` (Python bindings)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
