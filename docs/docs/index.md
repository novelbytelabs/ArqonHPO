---
hide:
  - navigation
  - toc
---

<style>
  .md-content__button { display: none; }
  .tx-hero {
    text-align: center;
    padding: 6rem 1rem;
    background: radial-gradient(circle at center, rgba(124, 58, 237, 0.15) 0%, transparent 70%);
  }
  .tx-hero h1 {
    font-weight: 800;
    font-size: 3.5rem;
    line-height: 1.1;
    margin-bottom: 1.5rem;
    background: -webkit-linear-gradient(315deg, #f3f4f6 0%, #9ca3af 74%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .tx-hero h2 {
    font-weight: 400;
    font-size: 1.5rem;
    color: #94a3b8;
    max-width: 800px;
    margin: 0 auto 2.5rem;
    line-height: 1.6;
  }
  .tx-buttons {
    display: flex;
    gap: 1rem;
    justify-content: center;
    margin-top: 2rem;
  }
  .md-button--primary {
    background-color: #7c3aed !important;
    border-color: #7c3aed !important;
    font-weight: 600;
    padding: 0.75rem 2rem;
    font-size: 1.1rem;
  }
  .md-button--secondary {
    font-weight: 600;
    padding: 0.75rem 2rem;
    font-size: 1.1rem;
  }
  .tx-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2rem;
    padding: 4rem 0;
  }
  .tx-card {
    background: rgba(30, 41, 59, 0.5);
    border: 1px solid rgba(148, 163, 184, 0.1);
    border-radius: 12px;
    padding: 2rem;
    transition: transform 0.2s, border-color 0.2s;
  }
  .tx-card:hover {
    transform: translateY(-4px);
    border-color: #7c3aed;
  }
  .tx-card h3 {
    margin-top: 0;
    font-weight: 700;
  }
</style>

<div class="tx-hero">
  <h1>Microsecond-budget optimization<br>for running systems.</h1>
  <h2>ArqonHPO is the control plane for self-tuning infrastructure. Embed adaptive optimization into your hot path with zero latency penalty.</h2>
  
  <div class="tx-buttons">
    <a href="documentation/quickstart/" class="md-button md-button--primary">Get Started</a>
    <a href="product/" class="md-button">Why ArqonHPO?</a>
  </div>
</div>

<div class="tx-grid">
  <div class="tx-card">
    <h3>🚀 Sub-Microsecond</h3>
    <p>Decision latency under 1μs (p99). Engineered for the tightest inner loops where milliseconds are an eternity. No allocations, lock-free audit, pure Rust core.</p>
  </div>
  <div class="tx-card">
    <h3>🛡️ Production Grade</h3>
    <p>Safety barriers, automated rollback, and bounded change rates. The <strong>SafetyExecutor</strong> (Tier 1) ensures your system never drifts into unstable states.</p>
  </div>
  <div class="tx-card">
    <h3>🧠 Adaptive Engine</h3>
    <p>Detects landscape structure automatically. Switches between structured search (Nelder-Mead) and chaotic exploration (TPE) on the fly.</p>
  </div>
</div>

# Why use ArqonHPO?

Most HPO tools are built for offline model training (days/hours). ArqonHPO is built for **online systems** (ms/µs).

| Feature | Standard HPO (Optuna/Ray) | ArqonHPO (Tier 1/2) |
| :--- | :--- | :--- |
| **Typical Latency** | 1ms - 100ms | **100ns - 1µs** |
| **Fail Mode** | Retry / Crash | **Fail Closed / Rollback** |
| **State** | DB / Redis | **Lock-free Atomic** |
| **Language** | Python | **Rust + FFI + Bindings** |
| **Safety** | None (User logic) | **Enforced Guardrails** |

## Benchmarks at a Glance

!!! quote "Speed is King"
    When evaluations are cheap (<10ms), volume beats intelligence. ArqonHPO runs **30,000 trials** in the time standard Python solvers run 100.

[View Full Benchmark Report](benchmarks/index.md){ .md-button }

## Ready for Production

Systems engineering requires trust. ArqonHPO is built on a strict **Constitution**:

*   **No Happy Path Testing**: Adversarial, fuzz, and chaos testing are mandatory.
*   **No Silent Failures**: All errors are typed, propagated, or handled with failsafes.
*   **Documentation First**: Every feature starts with a Spec and a Contract.

[Read the Constitution](project/about.md){ .md-button .md-button--secondary }
