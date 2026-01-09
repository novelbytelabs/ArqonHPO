# Test Coverage

This document covers code coverage strategy, current status, and identified gaps for the ArqonHPO project.

## Quick Reference

| Metric | Value | Date |
|--------|-------|------|
| **Overall Line Coverage** | 79% | Jan 2026 |
| **Total Tests** | ~290+ | |
| **Files at 100%** | 5 | |

---

## How to Measure Coverage

### Prerequisites

Install the coverage tooling:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Install LLVM tools (required)
rustup component add llvm-tools-preview
```

### Running Coverage

=== "Quick Summary"

    ```bash
    # Summary table (fastest)
    cargo llvm-cov --workspace --exclude ship
    ```

=== "HTML Report"

    ```bash
    # Generate HTML report
    cargo llvm-cov --workspace --exclude ship --html --open
    ```

=== "Show Missing Lines"

    ```bash
    # Show uncovered lines
    cargo llvm-cov --workspace --exclude ship --show-missing-lines
    ```

=== "JSON Export"

    ```bash
    # Export for CI integration
    cargo llvm-cov --workspace --exclude ship --json --output-path coverage.json
    ```

---

## Current Coverage Status

### Files at 100% Coverage ✅

| File | Tests | Notes |
|------|-------|-------|
| `config.rs` | 18 | Unit interval arithmetic, Domain, Scale |
| `lib.rs` | 1 | Re-exports and add function |
| `rng.rs` | 1 | RNG wrapper |
| `ffi/lib.rs` | 2 | FFI bindings |
| `telemetry.rs` | 12 | Ring buffer, digest validation |

### High Coverage (80%+) 🟢

| File | Coverage | Tests | Key Functionality |
|------|----------|-------|-------------------|
| `control_safety.rs` | 97% | 15 | SafeMode, thrashing detection, budget tracking |
| `orchestrator.rs` | 85% | 11 | AdaptiveEngine, SpsaProposer |
| `executor.rs` | 90% | 10 | Guardrails, RollbackPolicy |
| `spsa.rs` | 91% | 8 | SPSA optimization, trimmed mean |
| `audit.rs` | 94% | 6 | Audit event logging |
| `classify.rs` | 89% | 8 | Landscape classification |
| `tpe.rs` | 83% | 6 | Tree-structured Parzen Estimator |
| `probe.rs` | 79% | 12 | Prime index probe, uniform probe |

### Medium Coverage (60-80%) 🟡

| File | Coverage | Gap Lines | Improvement Path |
|------|----------|-----------|------------------|
| `main.rs` (CLI) | 78% | 327 | TUI/dashboard server loops, stdin interactive |
| `homeostasis.rs` | 77% | 5 | Minor edge cases |
| `config_atomic.rs` | 79% | 27 | Atomic config updates |
| `nelder_mead.rs` | 67% | 235 | Complex state machine in `step()` |
| `machine.rs` | 72% | 105 | `ask()` phase transitions |
| `multi_start_nm.rs` | 63% | 95 | Coordinate descent, phase transitions |

### Low Coverage (<60%) 🔴

| File | Coverage | Gap Lines | Root Cause |
|------|----------|-----------|------------|
| `dashboard.rs` | 56% | 51 | HTTP server, TUI rendering |
| `proposer.rs` | 83% | 1 | Single enum variant |

---

## Coverage Gaps Analysis

### 1. Interactive/Server Code

**Files:** `main.rs`, `dashboard.rs`  
**Gap:** ~378 lines combined

These functions require running servers or reading from stdin:

- `run_command` - Executes scripts in a loop
- `interactive_command` - Reads JSON commands from stdin
- `tui_command` - Terminal UI with crossterm
- `dashboard_command` - HTTP server + TUI

**Recommendation:** These are better tested via integration tests or E2E tests rather than unit tests.

### 2. Nelder-Mead State Machine

**File:** `nelder_mead.rs`  
**Gap:** 235 lines in `step()` function

The `step()` function is a complex state machine with many branches:

- `Init` → build simplex
- `Reflection` → expansion, contraction, or shrink
- `Expansion`, `OutsideContraction`, `InsideContraction`
- `Shrink` → evaluate all shrunk points
- `CoordinatePrepass` → greedy descent
- `SimplexBuild` → waiting for vertex evaluations

**Recommendation:** Create integration tests that drive the full optimization loop with known test functions (Rosenbrock, Sphere, Rastrigin).

### 3. Multi-Start Coordination

**File:** `multi_start_nm.rs`  
**Gap:** 95 lines in `step()` and `run_coordinate_descent()`

Multi-start NM coordinates multiple Nelder-Mead instances:

- Phase transitions: `CoordinateDescent` → `Triage` → `Commit`
- Stall detection and start switching
- Best start selection

**Recommendation:** Test with multi-modal test functions to exercise start switching.

### 4. Solver Phase Transitions

**File:** `machine.rs`  
**Gap:** 105 lines in `ask()` method

The solver's `ask()` method manages phase transitions:

- `Probe` → `Classify` → `Refine(Landscape)` → `Done`
- Strategy initialization based on landscape

**Recommendation:** Create end-to-end optimization tests that run through all phases.

---

## Pre-Push Coverage Checklist

Before pushing changes, verify coverage hasn't regressed:

```bash
# 1. Run all tests
cargo test --workspace --exclude ship

# 2. Check coverage summary
cargo llvm-cov --workspace --exclude ship

# 3. Verify no new 0% files
cargo llvm-cov --workspace --exclude ship 2>&1 | grep "0.00%"

# 4. (Optional) Generate HTML report for detailed review
cargo llvm-cov --workspace --exclude ship --html --open
```

---

## Coverage Improvement Plan

### Phase 1: Unit Tests (Complete ✅)

- [x] `config.rs` - 0% → 100%
- [x] `lib.rs` - 0% → 100%
- [x] `orchestrator.rs` - 0% → 85%
- [x] `control_safety.rs` - 68% → 97%
- [x] `telemetry.rs` - 73% → 100%

### Phase 2: Strategy Tests (In Progress)

- [x] `nelder_mead.rs` - 48% → 67%
- [x] `multi_start_nm.rs` - 34% → 63%
- [ ] Target: 80%+ with integration tests

### Phase 3: CLI Integration Tests (Planned)

- [ ] `ask_command` E2E
- [ ] `tell_command` E2E
- [ ] `export`/`import` round-trip

### Phase 4: Interactive Code (Deferred)

- [ ] TUI tests with mock terminal
- [ ] Dashboard tests with mock HTTP client
- [ ] Interactive mode with stdin pipes

---

## CI Integration

Coverage is automatically measured in CI via Codecov. The GitHub Actions workflow:

1. Runs `cargo llvm-cov` on each PR
2. Uploads results to Codecov
3. Posts coverage diff as PR comment

See [CI/CD Runbook](ci_cd.md) for workflow details.
