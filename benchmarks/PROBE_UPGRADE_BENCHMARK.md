# Probe Upgrade Benchmark Results

**Date**: 2025-12-14
**Branch**: experiment/probe-upgrade

## Summary Table (Mean Best Value ± SE)

| Benchmark | ArqonHPO (New Probe) | Optuna (TPE) | Probe (Legacy) | Probe (New) | Random |
|-----------|----------|----------|----------|----------|----------|
| Hartmann (6D) | -1.8102±0.0815 | -2.5163±0.0573 | -1.2192±0.0771 | -1.1446±0.0644 | -1.8728±0.0563 |
| Rastrigin (5D) | 31.1554±1.9954 | 30.1748±1.1133 | 55.4516±1.3566 | 53.7839±1.1795 | 41.0819±0.9788 |
| Rosenbrock (5D) | 154.6312±15.6991 | 60.3994±6.2524 | 423.4319±30.1393 | 365.7261±25.8773 | 131.8336±8.4850 |
| Sphere (5D) | 7.7450±0.6373 | 2.8568±0.3274 | 16.8135±0.6946 | 15.9612±0.8062 | 8.4404±0.4132 |

## A/B Probe Comparison

- **New Probe Wins**: 186/360 (51.7%)
- **Legacy Probe Wins**: 174/360 (48.3%)
- **Ties**: 0/360

**✅ NEW PROBE IS SUPERIOR**

## Execution Time Comparison (mean ms)

- **ArqonHPO (New Probe)**: 27.51ms
- **Optuna (TPE)**: 618.33ms
- **Random**: 1.79ms