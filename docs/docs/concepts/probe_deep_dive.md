# Deep Dive: The Prime-Golden Probe

The **PCR Probe** uses a deterministic "Low Discrepancy Sequence" called the **Prime-Golden Lattice**. 

This section explains the mathematics behind how ArqonHPO samples the search space to maximize information gain while avoiding the pitfalls of random sampling and rigid grids.

## The Problem

1.  **Random Sampling (Monte Carlo)**:

    - **Issue**: It "clumps". You often get points very close to each other (wasted effort) and large empty voids (missed information).
    - **Result**: Inefficient coverage of the landscape.

2.  **Grid Sampling**:
   
    - **Issue**: It suffers from the "Curse of Dimensionality". The number of points needed grows exponentially (`10^d`).
    - **Issue**: It aliases. If the underlying function has a period that matches the grid, you miss the structure entirely.

## The Solution: Prime-Golden Lattice

PCR uses a hybrid approach that combines **Prime Number Phases** (to break harmonics) with **Golden Ratio Sweeps** (to ensure even spacing).

### The Math

For the `i`-th sample in dimension `d`:

```
sample[i][d] = ( primes[i] / 1000 + φ * (d + 1) * i / N ) % 1.0
```

Where:

- `p_i`: The `i`-th prime number (used as a pseudo-random phase seed).
- `φ ≈ 0.618033...`: The Golden Ratio conjugate (the most irrational number, preventing harmonic aliasing).
- `d`: The dimension index (0, 1, 2...).
- `N`: The total number of planned samples.

### Visual Proof

This unique combination creates a **Non-Linear Lattice**. It looks random (no obvious repeating pattern) but has **Low Discrepancy** (no large holes or tight clumps).

![Probe Lattice Pattern](../assets/probe_lattice.png)

*Comparison of PCR Probe (Blue) vs Uniform Random (Red). Note how the Blue dot spread is more uniform, covering the edges and center without clumping, whereas Red leave large white gaps.*

## Why It Matters

This high-quality data is critical for the **Classifier** phase.
- If we used random points, we might accidentally sample a "flat" region of a structured function and misclassify it as chaotic.
- By using the Prime-Golden Lattice, `ResidualDecayClassifier` gets a statistically representative view of the landscape's global structure in just `N` evaluations.
