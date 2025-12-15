# Ablation Benchmark Results

**Date**: 2025-12-14

## A) Hold Refinement (NM), Swap Probes

| Function | Legacy Probe | New Probe | Random Probe | Best |
|----------|-------------|-----------|--------------|------|
| Sphere (5D) | 5.0180 | 0.0572 | 0.2166 | New |
| Rosenbrock (5D) | 7.7261 | 1.9937 | 10.0426 | New |
| Rastrigin (5D) | 40.4166 | 22.9390 | 20.9712 | Random |

## B) Hold Probe (New), Swap Refinement

| Function | None | NM | TPE | Random | Best |
|----------|------|----|----|--------|------|
| Sphere (5D) | 8.9708 | 0.0572 | 0.7808 | 7.2924 | NM |
| Rosenbrock (5D) | 68.5575 | 1.9937 | 10.9277 | 67.5879 | NM |
| Rastrigin (5D) | 47.8795 | 22.9390 | 20.3494 | 44.0249 | TPE |

## Key Findings

- **Probe contribution**: New probe is 72.1% better than Legacy on average
- **Refinement gap**: NM is 0.5× worse than TPE on average

**✅ CONCLUSION: The probe upgrade is working well**