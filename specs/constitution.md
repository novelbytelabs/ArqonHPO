# ArqonHPO Constitution

**Version**: 1.0  
**Status**: Active  
**Last Updated**: 2025-12-14

This document codifies the core principles that guide ArqonHPO development. Changes to this constitution require explicit approval and documentation of rationale.

---

## Core Principles

### 1. Determinism is Non-Negotiable

Same inputs → identical outputs. No exceptions.

- All randomness MUST come from an explicit seeded RNG.
- Tie-breaking MUST be stable and reproducible.
- Environment fingerprints MUST be captured for reproducibility.

### 2. No Magic Numbers Without Justification

Every constant MUST have documented mathematical or empirical basis.

- Bad: `let n = 1000.0; // Base resolution`
- Good: `prime_offset: 50  // Ensures separation from first ~50 primes used elsewhere`

### 3. Anytime Algorithms Preferred

Algorithms SHOULD NOT depend on knowing total N upfront.

- Prefixes MUST be meaningful.
- Early termination MUST yield valid results.

### 4. Test Quality, Not Just API

Include quality tests that verify mathematical properties:

- Discrepancy metrics for samplers
- Coverage tests for optimization
- Collision detection for sampling

### 5. Validate with Torus-Shift Protocol

Prevent "lucky alignment" artifacts in benchmarks:

- Use random shifts: `f_δ(x) = f((x + δ) mod 1)`
- Multiple seeds per shift for stochastic methods
- Report mean ± SE across shifts

---

## Sampling Standards

For probe samplers:

| Requirement | Rationale |
|-------------|-----------|
| MUST NOT use quantized phase terms (e.g., `/1000`) | Causes collisions |
| MUST use per-dimension independent phases | Prevents striping |
| SHOULD use irrational multipliers for phases | Low discrepancy |
| SHOULD be anytime (no N-dependence) | Flexibility |

---

## Architecture Principles

### Rust Core, Thin Bindings

- Core logic MUST be in Rust crate
- Python bindings MUST be thin wrappers (PyO3)
- Objective functions cross the FFI boundary, not optimizer state

### Pipeline is Sacred

Default behavior MUST follow:

```
probe → classify → mode select → refine
```

No bypass in default runs. User can opt-out explicitly.

### Artifacts are Contracts

- Schema-versioned JSON output
- Environment fingerprint included
- Replayable from artifacts

---

## Lessons Learned

### Probe Upgrade (2025-12-14)

**Problem**: Original `p/1000` prime phase caused:
- 32.6% collision rate at N=512
- Cross-dimension striping artifacts
- Worse discrepancy than random sampling

**Solution**: Prime-Sqrt-Slopes-Rot (Kronecker sequence)
- Irrational slopes from `√prime`
- Per-dimension prime-based rotations
- Validated via torus-shift protocol

**Principle Codified**: "Primes do real work" — primes should define geometry, not just offset phases.

---

## Amendment Process

1. Propose change via PR with rationale
2. Demonstrate empirical evidence if applicable
3. Update this document with lesson in "Lessons Learned" section
4. Update GEMINI.md memory file if needed
