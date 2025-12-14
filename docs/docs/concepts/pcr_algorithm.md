# PCR (Probe-Classify-Refine) Algorithm

The **PCR Algorithm** is ArqonHPO's core innovation (v2.0) that automatically selects the optimal optimization strategy based on the landscape's structure.

It solves the "Algorithm Selection Problem" by treating it as a classification task rather than a trial-and-error process.

## 1. Probe (Prime-Index Sampling)

The algorithm begins by sampling the landscape using a deterministic **Prime-Index Probe**. 
Instead of random sampling, it uses prime number ratios to generate a low-discrepancy sequence that covers multiple scales simultaneously.

- **Goal**: Gather enough data to estimate the landscape's "roughness".
- **Method**: Evaluate `N` points (configurable, default 20% of budget).

## 2. Classify (Residual Decay Analysis)

The algorithm analyzes the probe data using a **ResidualDecayClassifier**. It looks at how the best-so-far value improves over time.

- **Structured Landscapes** (Smooth, Convex-ish):
    - Exhibit **geometric decay** in residuals.
    - Estimates characteristic decay exponent `α > 0.5`.
    - **Classification**: `Structured`.

- **Chaotic Landscapes** (Noisy, Multimodal, Discontinuous):
    - Exhibit **irregular/flat** residuals.
    - Estimates `α ≤ 0.5`.
    - **Classification**: `Chaotic`.

## 3. Refine (Strategy Selection)

Based on the classification, the solver switches to the optimal refinement strategy:

| Classification | Strategy | Why? |
|----------------|----------|------|
| **Structured** | **Nelder-Mead** | Exploits gradients/structure for extremely fast convergence (up to 300x faster than TPE). |
| **Chaotic** | **TPE** | Uses probabilistic modeling (Tree-structured Parzen Estimator) to navigate noise and local optima robustly. |

### Warm-Starting (Top-K Seeding)

The refinement phase is **warm-started** using the best points found during the Probe phase.
- **Nelder-Mead**: Initial simplex is constructed around the best probe points.
- **TPE**: All probe history is used to build the initial density kernel.

## Benefits

- **Speed**: On structured problems (simulations), you get Nelder-Mead speeds.
- **Robustness**: On noisy problems (ML tuning), you get TPE stability.
- **Zero Config**: No need to choose an algorithm; PCR adapts automatically.
