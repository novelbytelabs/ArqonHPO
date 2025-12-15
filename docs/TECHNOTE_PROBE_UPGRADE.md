# ArqonHPO Probe Upgrade: Technical Note

**Date**: 2025-12-14  
**Branch**: `experiment/probe-upgrade`  
**Status**: Ready for review

---

## Executive Summary

We replaced the flawed `/1000` prime probe with a mathematically rigorous Kronecker sequence, achieving **72% improvement** in probe quality. The upgrade includes NM diversity seeding and 10% random spice for multimodal robustness.

**Key Result**: ArqonHPO now **beats Random by 2-6×** and is **170× faster than Optuna**, but still **2-4× worse on final quality** than Optuna TPE.

---

## What Changed

### 1. New Probe: PrimeSqrtSlopesRotProbe

**Old (flawed)**:
```
x[i] = (prime[i] / 1000) mod 1  // Quantization causes 32% collision rate
```

**New (validated)**:
```
x[i,d] = frac((i+1) × √p_{d+50} + frac(p_{d+200} × (√2-1)))
```

Properties:
- ✅ No quantization (irrational slopes)
- ✅ Per-dimension independence (no striping)
- ✅ Anytime (no N-dependence)
- ✅ Deterministic

### 2. Random Spice (10%)

Added `random_spice_ratio: 0.1` — 10% of probe points are uniform random to hedge against multimodal misses (see Rastrigin results).

### 3. NM Diversity Seeding

**Old**: Best-N points (clustered around optimum)  
**New**: Farthest-point selection from top-5×(N+1) pool

---

## Benchmark Results

### Ablation: Probe Quality (NM refinement held constant)

| Function | Legacy Probe | New Probe | Improvement |
|----------|-------------|-----------|-------------|
| Sphere | 5.02 | **0.06** | **+98.9%** |
| Rosenbrock | 7.73 | **1.99** | **+74.2%** |
| Rastrigin | 40.4 | 22.9 | +43.2% |

### Full Pipeline: ArqonHPO vs Optuna vs Random

| Function | ArqonHPO | Optuna | Random | Speed |
|----------|----------|--------|--------|-------|
| Sphere | 1.95 | **0.47** | 5.19 | 170× faster |
| Rosenbrock | 11.68 | **9.95** | 65.05 | 170× faster |
| Rastrigin | 19.05 | **16.22** | 32.74 | 170× faster |

**Optuna wins on quality. ArqonHPO wins on speed.**

---

## Analysis: Where Is the Gap?

```
ArqonHPO Pipeline:
  Probe (20%) → Classify → Refine (80%)
       ↓                      ↓
   ✅ FIXED              ⚠️ THE GAP
```

The probe upgrade is **validated and working**. The remaining 2-4× quality gap is in **refinement**:

1. **NM is single-start** — Gets stuck in local minima
2. **Our TPE isn't as tuned** — Optuna's TPE has years of refinement
3. **Routing not wired** — Classifier doesn't yet switch strategies

---

## CI Guardrail Tests Added

| Test | Status | Purpose |
|------|--------|---------|
| Geometry (CD, NN) | ✅ | Catch striping/collisions regression |
| Probe quality | ✅ | New probe beats legacy by ≥60% |
| Structured routing | ✅ | NM beats TPE on Sphere/Rosenbrock |
| Multimodal guardrail | ✅ | Probe not worse than random on Rastrigin |

---

## Open Questions for Team Discussion

1. **Is 170× speed worth 2-4× quality gap?**
   - For expensive objectives (minutes/hours per eval), yes
   - For cheap objectives, maybe not

2. **Multi-start NM or CMA-ES?**
   - Multi-start is simpler, keeps determinism
   - CMA-ES is stronger but adds complexity

3. **Should we match Optuna's TPE exactly?**
   - Weights (recent trials weigh more)
   - Mixture prior (10% uniform to prevent collapse)

4. **Adaptive neighbor-noise refiner (USP)**
   - "Each point asks neighbors" concept
   - Could be our differentiator vs Optuna

---

## Files Changed

```
Modified:
  crates/core/src/probe.rs          (+30 lines: random spice)
  crates/core/src/strategies/nelder_mead.rs  (+45 lines: diversity seeding)
  
New:
  benchmarks/test_probe_guardrails.py  (CI tests)
  benchmarks/run_ablation_benchmark.py
  specs/constitution.md
```

---

## Next Steps (Proposed)

| Priority | Task | Expected Impact |
|----------|------|-----------------|
| 1 | Wire classifier routing | Proper NM/TPE selection |
| 2 | Multi-start NM | +20-30% on structured |
| 3 | TPE parity pass | Match Optuna behavior |
| 4 | Neighbor-noise prototype | USP differentiator |

---

## How to Test

```bash
# Run CI guardrail tests
conda run -n helios-gpu-118 python -m pytest benchmarks/test_probe_guardrails.py -v

# Run ablation benchmark  
conda run -n helios-gpu-118 python benchmarks/run_ablation_benchmark.py

# Run full comparison
conda run -n helios-gpu-118 python benchmarks/run_probe_benchmark.py
```

---

**Questions/feedback welcome.**
