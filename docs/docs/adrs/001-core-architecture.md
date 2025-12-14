# ADR-001: Core Architecture

**Status:** Accepted  
**Date:** 2024-12-13

## Context

We need an HPO library that:

1. Automatically selects between optimization strategies.
2. Is deterministic and reproducible.
3. Has a high-performance core with Python bindings.

## Decision

We implement a **Rust Core with PyO3 Bindings** architecture:

```
┌─────────────┐     ┌─────────────────────────────────────┐
│   Python    │────▶│          arqonhpo._internal         │
│   Client    │     │             (PyO3)                  │
└─────────────┘     └────────────────┬────────────────────┘
                                     │
                    ┌────────────────▼────────────────────┐
                    │          arqonhpo-core              │
                    │  ┌─────────────────────────────┐   │
                    │  │    Solver State Machine     │   │
                    │  │  Probe→Classify→Refine      │   │
                    │  └─────────────────────────────┘   │
                    │  ┌─────────────────────────────┐   │
                    │  │       Strategies            │   │
                    │  │  ▪ NelderMead (Structured)  │   │
                    │  │  ▪ TPE (Chaotic)            │   │
                    │  └─────────────────────────────┘   │
                    └─────────────────────────────────────┘
```

### Key Principles

1. **Probe-Gated Pipeline:** All runs start with a deterministic probe phase. Classification happens *after* probing, preventing premature strategy commitment.

2. **Seed Sovereignty:** All randomness flows from explicit seeds. No hidden global RNG.

3. **Strategy Trait:** Extensible via `impl Strategy for YourOptimizer`.

## Consequences

- **Pro:** Single codebase for CLI, FFI, and Python.
- **Pro:** Deterministic by design.
- **Con:** Requires Rust toolchain to build from source.
