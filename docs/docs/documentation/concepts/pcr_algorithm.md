# PCR (Probe-Classify-Refine) Algorithm

### **The core of ArqonHPO: automatic algorithm selection based on your problem's structure.**

The **PCR Algorithm** is ArqonHPO's core innovation (v2.0) that automatically selects the optimal optimization strategy based on the landscape's structure. It solves the "Algorithm Selection Problem" by treating it as a classification task rather than a trial-and-error process.

Instead of requiring you to choose between different optimization algorithms (which requires expertise), PCR analyzes your problem, determines what kind of landscape it presents, and selects the best algorithm automatically.

---

## The Three Phases

```mermaid
graph LR
    A[📍 Probe] --> B[🔬 Classify]
    B --> C[🎯 Refine]
    C --> D[✅ Done]

    style A fill:#6366f1,color:#fff
    style B fill:#8b5cf6,color:#fff
    style C fill:#a855f7,color:#fff
    style D fill:#22c55e,color:#fff
```

| Phase        | What Happens                                | Budget Used           |
| ------------ | ------------------------------------------- | --------------------- |
| **Probe**    | Sample the landscape systematically         | ~20%                  |
| **Classify** | Analyze samples to determine landscape type | 0% (computation only) |
| **Refine**   | Run the best algorithm for your problem     | ~80%                  |

---

## Phase 1: Probe (Prime-Index Sampling)

The algorithm begins by sampling the landscape using a deterministic **Prime-Index Probe**.

### What It Does

Instead of random sampling, the probe uses prime number ratios to generate a **low-discrepancy sequence**. This mathematical technique ensures samples are spread evenly across the search space, covering multiple scales simultaneously.

Random sampling can cluster points together by chance. Low-discrepancy sampling guarantees uniform coverage, avoiding wasted evaluations on redundant areas.

### Why Prime Numbers?

Prime numbers have a special property: their ratios are irrational and "maximally non-repeating." This creates sampling patterns that:

- Cover the space more uniformly than random sampling
- Avoid grid-like artifacts that could miss important regions
- Are fully deterministic and reproducible

### Configuration

```python
config = {
    "budget": 100,
    "probe_ratio": 0.2,  # 20% of budget for probing (default)
    # ...
}
```

- **Goal**: Gather enough data to estimate the landscape's "roughness"
- **Method**: Evaluate `N` points (configurable, default 20% of budget)
- **Result**: A collection of (parameters → objective value) samples

---

## Phase 2: Classify (Residual Decay Analysis)

After probing, the algorithm analyzes the collected data to determine the landscape type.

### The Core Question

> "Does this problem have exploitable structure, or is it essentially noisy and chaotic?"

### How It Works: Residual Decay

The **ResidualDecayClassifier** looks at how the best-so-far value improves as more samples are collected. Specifically, it measures the **decay rate** (α) of the residuals.

**What are residuals?** The differences between consecutive objective values when sorted from best to worst.

**The key insight:**

- **Structured functions** (smooth, bowl-shaped): Values near the optimum are densely packed. Residuals decay geometrically—each step gets you proportionally closer.
- **Chaotic functions** (noisy, multimodal): Values are scattered unpredictably. Residuals don't follow a consistent pattern.

### The Math (Simplified)

Given residuals E₁, E₂, E₃, ..., we fit an exponential decay curve:

```
E_k ≈ C × β^k
```

Where:

- `β` is the decay factor (0 < β < 1 for decay)
- `α = -ln(β)` is the **decay rate**

### Classification Rules

The classifier uses a threshold of **α = 0.5** (configurable). This threshold was chosen empirically based on testing across commonly used optimization benchmark functions.

| Decay Rate (α) | Residual Pattern  | Classification | What It Means                                                           |
| -------------- | ----------------- | -------------- | ----------------------------------------------------------------------- |
| **α > 0.5**    | Geometric decay   | **Structured** | Residuals decrease quickly—there's an exploitable gradient-like pattern |
| **α ≤ 0.5**    | Flat or irregular | **Chaotic**    | Residuals don't decay consistently—the landscape is noisy or multimodal |

### What You See

When running ArqonHPO, you'll see output like:

```
[Machine] Classified as Structured (Score: 1.0172)
```

The score is the estimated α value. Higher scores mean more structure.

---

## Phase 3: Refine (Strategy Selection)

Based on the classification, the solver switches to the optimal refinement strategy:

| Classification | Strategy Selected                          | Why This Works                                                                                                      |
| -------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| **Structured** | **Nelder-Mead**                            | A derivative-free simplex method that exploits smooth, bowl-shaped landscapes for efficient convergence             |
| **Chaotic**    | **TPE** (Tree-structured Parzen Estimator) | A probabilistic model-based method that handles noise and multiple local optima by modeling "good" vs "bad" regions |

### Warm-Starting (Top-K Seeding)

The refinement phase doesn't start from scratch. It's **warm-started** using the best points found during the Probe phase:

- **Nelder-Mead**: Initial simplex is constructed around the best probe points
- **TPE**: All probe history is used to build the initial density kernel

**Why this matters:** You don't waste the 20% of budget spent on probing. Those samples inform the refinement strategy, giving it a head start.

### Fail-Safe Mechanisms

Sometimes the classifier's initial assessment is wrong, or the landscape changes character in different regions. ArqonHPO includes fail-safes:

```
[Machine] Structured Fail-Safe Triggered! Restarting with CP Shift at param count 36
```

**What this means:** If Nelder-Mead (structured strategy) isn't making progress, the solver can restart with a shifted center point or switch strategies.

---

## Why PCR Matters

### The Algorithm Selection Problem

Traditional HPO requires you to choose:

- Random Search? (simple but inefficient)
- Bayesian Optimization? (good for noisy, expensive)
- Nelder-Mead? (fast for smooth functions)
- CMA-ES? (good for high dimensions)

**The problem:** Choosing the wrong algorithm can result in significantly more evaluations than necessary, or worse, failure to converge at all.

### PCR's Approach

PCR invests a portion of the budget (default 20%) to _measure_ your problem's characteristics, then makes an informed strategy selection. The key advantages:

- **Automatic adaptation**: No manual algorithm selection required
- **Warm-started refinement**: Probe samples seed the refinement strategy, so the probing budget isn't wasted
- **Fail-safe mechanisms**: If the initial classification appears incorrect, the solver can adapt

---

## Example Walkthrough

Let's trace through what happens when you run a simple optimization:

```python
config = {
    "seed": 42,
    "budget": 50,
    "bounds": {"x": {"min": -10, "max": 10}, "y": {"min": -10, "max": 10}}
}
```

### Step 1: Probe Phase

With `probe_ratio=0.2` and `budget=50`, the solver allocates 10 evaluations for probing.

- Samples 10 points using Prime-Index probe
- Evaluates your objective function at each point
- Collects results: `[(x₁, y₁) → 5.2, (x₂, y₂) → 3.1, ...]`

### Step 2: Classify Phase

Analyzes the 10 samples:

- Sorts objective values: `[0.8, 1.2, 2.1, 3.1, ...]`
- Computes residuals: `[0.4, 0.9, 1.0, ...]`
- Fits exponential decay, estimates α = 1.02
- **Classification: Structured** (α > 0.5)

Output: `[Machine] Classified as Structured (Score: 1.0172)`

### Step 3: Refine Phase

Switches to Nelder-Mead:

- Initializes simplex around best probe points
- Runs Nelder-Mead with remaining 40 evaluations
- Converges toward optimum

---

## When PCR Works Best

✅ **Ideal for:**

- Unknown problems where you don't know the landscape character
- Mix of smooth and noisy problems in a pipeline
- Budget-constrained optimization (can't afford trial-and-error)
- Automated systems where human expertise isn't available

⚠️ **Consider alternatives when:**

- You _know_ your problem is noisy (just use TPE directly via `ask_one()`)
- Very small budgets (< 20 evaluations) where probe overhead is too high
- Real-time control where batch-style probing isn't appropriate

---

## Tuning PCR

### Probe Ratio

```python
config = {
    "probe_ratio": 0.3,  # More probing for uncertain landscapes
    # ...
}
```

| Probe Ratio       | Trade-off                                                         |
| ----------------- | ----------------------------------------------------------------- |
| **0.1**           | Less exploration, faster to refinement, risk of misclassification |
| **0.2** (default) | Balanced                                                          |
| **0.3-0.4**       | More confident classification, less budget for refinement         |

### Bypassing PCR

For real-time/online optimization, use `ask_one()` which skips PCR entirely and uses TPE directly:

```python
candidate = solver.ask_one()  # No probe/classify, direct TPE
```

---

## Summary

| Aspect     | Description                                                                          |
| ---------- | ------------------------------------------------------------------------------------ |
| **What**   | Automatic algorithm selection based on measured landscape properties                 |
| **Why**    | Eliminates guesswork, adapts to your specific problem                                |
| **How**    | 20% budget on probing → classify via residual decay → 80% budget on optimal strategy |
| **Result** | Near-optimal performance on both structured AND chaotic problems                     |

---

## Next Steps

<div class="grid cards" markdown>

- :mag: **[Probe Deep Dive](probe_deep_dive.md)**

  Low-discrepancy sampling and prime-index mathematics

- :chart_with_upwards_trend: **[Strategies](strategies.md)**

  Detailed breakdown of Nelder-Mead and TPE

- :zap: **[Batch vs. Online](batch_vs_online.md)**

  When to use PCR vs. `ask_one()`

- :shield: **[Safety](safety.md)**

  How guardrails protect the refinement phase

</div>

---
