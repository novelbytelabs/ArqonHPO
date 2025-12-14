# ArqonHPO Iteration 3 - Brainstorming

## Current State Assessment

### ✅ What's Working
- **Speed**: 177x average speedup on time-to-target
- **Peak Performance**: 318x on Sphere-5D, 138x on Rosenbrock-2D
- **RPZL Algorithm**: Successfully implemented and merged to main
- **Infrastructure**: Tests, docs, benchmarks all in place

### ⚠️ Critical Gap: Target Reach Rate

| Benchmark | ArqonHPO | Optuna | Gap |
|-----------|----------|--------|-----|
| Sphere-2D | 100% | 100% | ✅ Tied |
| Sphere-5D | 60% | 80% | ❌ -20% |
| Rosenbrock-2D | 20% | 100% | ❌ -80% |
| Rastrigin-2D | 0% | 80% | ❌ -80% |
| Ackley-2D | 40% | 100% | ❌ -60% |

**Root Cause**: Algorithm quality, not speed.

---

## Improvement Ideas

### Category 1: Probe Phase Improvements

**Problem**: Low probe sample count may miss structure

**Ideas**:
1. **Adaptive Probe Budget**: Scale probe ratio with dimensionality (e.g., 0.2 + 0.05*dims)
2. **Latin Hypercube Sampling**: Replace random probe with LHS for better coverage
3. **Sobol Sequences**: Low-discrepancy sequences for uniform coverage
4. **Multi-Resolution Grid**: Combine prime-index with hierarchical grid sampling
5. **Probe Quality Gates**: Require minimum variance/diversity before proceeding

### Category 2: Classification Improvements

**Problem**: ResidualDecayClassifier may misclassify edge cases

**Ideas**:
1. **Ensemble Classifier**: Combine ResidualDecay + Variance + Gradient-based
2. **Adaptive Thresholds**: Learn α threshold from probe statistics
3. **Landscape Fingerprinting**: Multi-metric signature (α, CV, smoothness, modality)
4. **Confidence Scores**: Weight strategy selection by classification confidence
5. **Fallback Strategy**: Default to hybrid approach when uncertain

### Category 3: Nelder-Mead Enhancements

**Problem**: 20% success on Rosenbrock suggests poor initialization/convergence

**Ideas**:
1. **Better Simplex Initialization**: Use Top-K + gradient estimation for initial simplex
2. **Adaptive Coefficients**: Tune α,γ,ρ,σ during optimization (CMA-NM)
3. **Restart Strategy**: Multi-start from different probe seeds
4. **Gradient-Enhanced NM**: Use finite differences to guide reflection
5. **Trust Region**: Add bounds enforcement to prevent divergence
6. **Accelerated NM**: Use momentum/acceleration for faster convergence

### Category 4: TPE Improvements

**Problem**: 0% on Rastrigin suggests TPE isn't handling chaos well

**Ideas**:
1. **Better Kernel Bandwidth**: Adaptive bandwidth per dimension
2. **More Initial Samples**: Increase warmup samples for better density estimation
3. **Parzen Window Variants**: Try Gaussian processes or other kernel types
4. **Multi-Fidelity TPE**: Use cheap approximations to guide search
5. **Expected Improvement**: Switch from EI to other acquisition functions (UCB, PI)

### Category 5: Hybrid/Meta Strategies

**Problem**: Binary choice (NM or TPE) may miss opportunities

**Ideas**:
1. **Portfolio Approach**: Run multiple strategies in parallel, use best
2. **Sequential Hybrid**: NM for exploitation, TPE for exploration
3. **Budget Allocation**: Dynamically allocate trials between strategies
4. **Strategy Switching**: Allow mid-run strategy changes based on progress
5. **Ensemble Predictions**: Combine predictions from multiple models

### Category 6: Infrastructure/Tooling

**Ideas**:
1. **Benchmark Suite Expansion**: Add more diverse test functions
2. **Performance Profiling**: Identify bottlenecks in Rust code
3. **Hyperparameter Auto-Tuning**: Meta-optimize ArqonHPO's own parameters
4. **Online Learning**: Update classifier/strategy params during optimization
5. **Visualization Tools**: Real-time convergence plots for debugging

---

## Deep Research Prompt for Perplexity

```
I'm developing ArqonHPO, a Rust-based hyperparameter optimization library that's 177x faster than Optuna but has lower convergence quality on hard problems (Rosenbrock, Rastrigin).

Current architecture:
- Probe phase: PrimeIndexProbe samples landscape
- Classify phase: ResidualDecayClassifier estimates α from residual decay
  - α > 0.5 → Structured → Nelder-Mead
  - α ≤ 0.5 → Chaotic → TPE
- Refine phase: Top-K probe points seed chosen strategy

Benchmark Results (Time-to-Target):
- Sphere-2D: 134x faster, 100% success rate (tied with Optuna)
- Sphere-5D: 318x faster, 60% vs Optuna's 80%
- Rosenbrock-2D: 138x faster, 20% vs Optuna's 100%
- Rastrigin-2D: N/A, 0% vs Optuna's 80%
- Ackley-2D: 118x faster, 40% vs Optuna's 100%

Please research and recommend:

1. **State-of-the-art improvements to the Nelder-Mead algorithm** (2015-2024 papers)
   - Adaptive coefficient schemes
   - Better simplex initialization methods
   - Restart strategies for multimodal landscapes
   - Hybrid approaches combining NM with other methods

2. **Advanced TPE/Bayesian optimization techniques** for chaotic landscapes
   - Better kernel bandwidth selection beyond Scott's Rule
   - Acquisition function alternatives to Expected Improvement
   - Multi-fidelity optimization approaches
   - Handling of discrete/categorical parameters

3. **Landscape classification methods** better than α estimation from residuals
   - Machine learning approaches to landscape recognition
   - Multi-metric landscape signatures
   - Online learning during optimization

4. **Probe/sampling strategies** superior to prime-index sampling
   - Latin Hypercube Sampling variants
   - Sobol/Halton low-discrepancy sequences
   - Adaptive sampling based on initial results
   - Multi-resolution sampling schemes

5. **Meta-optimization and algorithm selection** literature
   - Portfolio approaches (running multiple algorithms)
   - Online algorithm selection during optimization
   - Budget allocation between exploration/exploitation

Focus on methods that:
- Have proven results on standard benchmarks (CEC, BBOB)
- Are implementable in Rust (no Python-specific libraries)
- Maintain low overhead (< 10ms per iteration)
- Work well with limited evaluation budgets (< 500 trials)

Include citations to key papers/repositories when available.
```

---

## Next Steps

1. **Submit research prompt to Perplexity** → Incorporate findings
2. **Create spec.md** for selected improvements
3. **Prioritize** based on impact/effort
4. **Plan implementation** following Spec-Kit workflow
