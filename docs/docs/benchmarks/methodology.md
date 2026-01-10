# Benchmark Methodology

This page documents how our benchmarks were conducted for reproducibility.

---

## Hardware

| Component | Specification                            |
| --------- | ---------------------------------------- |
| CPU       | AMD Ryzen 9 5950X (16 cores, 32 threads) |
| RAM       | 64 GB DDR4-3600                          |
| Storage   | NVMe SSD (Samsung 980 Pro)               |
| OS        | Ubuntu 22.04 LTS                         |

---

## Software Versions

| Software | Version |
| -------- | ------- |
| Rust     | 1.82.0  |
| Python   | 3.12.0  |
| ArqonHPO | 0.3.0   |
| Optuna   | 3.5.0   |
| NumPy    | 1.26.0  |

---

## Benchmark Configuration

### Time-Bounded Benchmarks

- **Budget:** 5 seconds wall-clock time
- **Warmup:** 1 second (excluded from measurement)
- **Iterations:** 5 runs, median reported
- **RNG Seed:** 42 (deterministic)

### Test Functions

| Function   | Dimensions | Characteristics       |
| ---------- | ---------- | --------------------- |
| Sphere     | 10         | Smooth, unimodal      |
| Rosenbrock | 10         | Smooth, narrow valley |
| Rastrigin  | 10         | Noisy, multimodal     |
| Ackley     | 10         | Noisy, multimodal     |

---

## Measurement Protocol

1. **Isolation:** Benchmarks run on dedicated hardware, no other processes
2. **CPU Governor:** Set to `performance` mode
3. **Hyperthreading:** Enabled
4. **Metrics:**
   - Trials completed per second
   - Time to best value
   - Final objective value

---

## Reproduction Steps

```bash
# Clone and build
git clone https://github.com/novelbytelabs/ArqonHPO.git
cd ArqonHPO
cargo build --release

# Run benchmarks
cargo bench --bench optimizer_comparison

# Compare with Optuna
python benchmarks/optuna_comparison.py
```

---

## Data Files

Raw benchmark data available in:

- `docs/docs/benchmarks/benchmark_data.csv`

---

## Limitations

- **Single machine:** Results may vary on different hardware
- **Python GIL:** Optuna measurements include Python overhead
- **Function cost:** Benchmarks use instant ($\approx 0$ms) functions

For expensive functions (>1s per evaluation), Optuna's smarter sampling may outperform ArqonHPO's volume advantage.
