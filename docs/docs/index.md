---
hide:
  - navigation
  - toc
---

<style>
  .md-content__button { display: none; }
  .md-sidebar--secondary { display: none; }
  .md-main__inner { margin-top: 0 !important; }
  
  /* Hero Section */
  .hero-section {
    position: relative;
    padding: 10rem 0 12rem; /* Increased padding */
    text-align: center;
    /* Stronger Glowing Orb Background */
    background: radial-gradient(circle at 50% 40%, rgba(99, 102, 241, 0.35) 0%, transparent 70%);
    margin: -2rem -2rem 0 -2rem !important;
    overflow: visible;
  }
  
  .hero-content {
    position: relative;
    z-index: 2;
    max-width: 1400px; /* Wider container */
    margin: 0 auto;
    padding: 0 2rem;
  }

  /* Text Shimmer Animation */
  @keyframes shimmer {
    0% { background-position: 0% 50%; }
    50% { background-position: 100% 50%; }
    100% { background-position: 0% 50%; }
  }

  .hero-title {
    font-size: 6.5rem !important; /* MASSIVE */
    font-weight: 900 !important;
    letter-spacing: -4px !important;
    margin-bottom: 2rem !important;
    line-height: 1.0 !important;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    
    /* High contrast gradient */
    background: linear-gradient(90deg, #ffffff 0%, #a5b4fc 40%, #ffffff 60%, #818cf8 100%);
    background-size: 200% auto;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    animation: shimmer 6s linear infinite;
    
    filter: drop-shadow(0 0 30px rgba(99, 102, 241, 0.3)); /* Text Glow */
  }
  
  .hero-subtitle {
    font-size: 2rem !important;
    color: #cbd5e1;
    max-width: 900px;
    margin: 0 auto 2rem;
    line-height: 1.3;
    font-weight: 400;
    text-shadow: 0 2px 10px rgba(0,0,0,0.5);
  }
  
  .hero-support {
    font-size: 1.25rem;
    color: #818cf8;
    margin-bottom: 2rem; /* Reduced bottom margin to fit engine badge */
    font-family: monospace;
    font-weight: 700;
    letter-spacing: 1px;
    text-transform: uppercase;
  }
  
  .hero-engine {
    font-size: 1rem;
    color: #94a3b8;
    margin-bottom: 4rem;
    font-family: 'Inter', sans-serif;
    opacity: 0.8;
  }
  .hero-engine code {
    background: rgba(99, 102, 241, 0.15);
    color: #a5b4fc;
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    border: 1px solid rgba(99, 102, 241, 0.3);
    font-family: monospace;
    font-weight: 600;
  }

  .cta-buttons {
    display: flex;
    gap: 2rem;
    justify-content: center;
    margin-bottom: 2rem;
  }
  
  .btn {
    padding: 1.2rem 3rem;
    border-radius: 50px;
    font-size: 1.2rem;
    font-weight: 700;
    text-decoration: none !important;
    transition: all 0.2s ease;
  }
  
  .btn-primary {
    background: #6366f1;
    color: white !important;
    border: 1px solid rgba(255,255,255,0.2);
    box-shadow: 0 0 30px rgba(99, 102, 241, 0.5);
  }
  
  .btn-primary:hover {
    background: #4f46e5;
    transform: translateY(-3px) scale(1.05);
    box-shadow: 0 0 50px rgba(99, 102, 241, 0.8);
  }
  
  .btn-secondary {
    background: rgba(15, 23, 42, 0.6);
    color: #e2e8f0 !important;
    border: 1px solid #475569;
    backdrop-filter: blur(10px);
  }
  
  .btn-secondary:hover {
    background: rgba(255,255,255,0.1);
    border-color: #cbd5e1;
    transform: translateY(-3px);
  }

  /* Pillars Grid */
  .grid-4 {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); /* Wider cards */
    gap: 2rem;
    margin: 4rem 0;
  }
  .card {
    background: #1e293b;
    padding: 3rem 2rem;
    border-radius: 16px;
    border: 1px solid #334155;
    transition: all 0.2s ease;
    height: 100%;
  }
  
  .card:hover {
    transform: translateY(-8px);
    border-color: #6366f1;
    box-shadow: 0 10px 40px -10px rgba(0,0,0,0.5);
  }
  .card-icon {
    font-size: 3rem;
    margin-bottom: 1.5rem;
  }
  .card h3 {
    margin-top: 0;
    font-size: 1.5rem;
    margin-bottom: 1rem;
    color: #f8fafc;
    font-weight: 700;
  }
  .card p {
    color: #94a3b8;
    line-height: 1.6;
    margin: 0;
    font-size: 1.05rem;
  }
  
  /* Mobile Responsive Fixes */
  @media (max-width: 768px) {
    .hero-title { font-size: 3.5rem !important; }
    .hero-subtitle { font-size: 1.25rem !important; }
    .hero-section { padding: 6rem 0; }
  }
</style>

<div class="hero-section">
  <div class="hero-content">
    <h1 class="hero-title">Arqon Runtime Optimizer</h1>
    <p class="hero-subtitle">Runtime optimization infrastructure for live production systems.</p>
    <p class="hero-support">Deterministic • Guardrailed • Sub-microsecond</p>
    <p class="hero-engine">Powered by <code>ArqonHPO</code></p>
    
    <div class="cta-buttons">
      <a href="documentation/quickstart/" class="btn btn-primary">Get Started</a>
      <a href="demos/" class="btn btn-secondary">See It Live</a>
    </div>
  </div>
</div>

<!-- The 4 Pillars -->
<div class="grid-4">
  <div class="card">
    <div class="card-icon">👻</div>
    <h3>Invisibility</h3>
    <p>Hot-path safe. The optimizer lives inside loops where "asking Python" is absurd. Zero allocations, nanosecond overhead.</p>
  </div>
  <div class="card">
    <div class="card-icon">🎯</div>
    <h3>Determinism</h3>
    <p>Replayable control. Same inputs + same seed = same decisions. Fully debuggable and auditable production behavior.</p>
  </div>
  <div class="card">
    <div class="card-icon">🛡️</div>
    <h3>Safety</h3>
    <p>Bounded exploration. First-class guardrails, rollback mechanisms, and strict delta limits ensure stability.</p>
  </div>
  <div class="card">
    <div class="card-icon">🌍</div>
    <h3>Ubiquity</h3>
    <p>Runs anywhere. Rust, C, C++, WASM, Embedded, Server. One optimization primitive for every target.</p>
  </div>
</div>

---

<div class="proof-section">
  <div class="proof-card">
    <div class="proof-content">
      <div style="color: #ffb300; font-weight: bold; margin-bottom: 0.5rem">PROOF A</div>
      <h2>Survive the Drift</h2>
      <p><strong>The Killer Demo.</strong> Use cases change. Traffic shifts. Hardware ages. A static "best config" degrades over time.</p>
      <p>ArqonHPO acts as a homeostatic system, continuously adapting to the new reality to maintain optimal performance.</p>
      <ul style="margin-top: 1rem; color: #94a3b8">
        <li>✅ Recovery from traffic spikes</li>
        <li>✅ Adaptation to data drift</li>
        <li>✅ Zero-downtime tuning</li>
      </ul>
    </div>
    <div class="proof-visual">
      <!-- Placeholder: To be replaced with real drifting chart -->
      <img src="benchmarks/throughput_comparison.png" alt="Drift Recovery Chart" style="opacity: 0.5">
      <div style="position:absolute">Drift Recovery Chart</div>
    </div>
  </div>

  <div class="proof-card">
    <div class="proof-content">
      <div style="color: #ffb300; font-weight: bold; margin-bottom: 0.5rem">PROOF B</div>
      <h2>Flatten the Jitter</h2>
      <p>Traditional optimizers introduce garbage collection pauses and latency spikes. ArqonHPO is designed for the hot path.</p>
      <p>We prove it by measuring the p99 latency overhead of the optimizer itself. It's effectively zero.</p>
      <ul style="margin-top: 1rem; color: #94a3b8">
        <li>✅ Sub-microsecond decision loops</li>
        <li>✅ No GC pauses</li>
        <li>✅ Predictable tail latency</li>
      </ul>
    </div>
    <div class="proof-visual">
        <div style="text-align: center">
            <h2>&lt; 1 µs</h2>
            <p>Decision Latency (p99)</p>
        </div>
    </div>
  </div>

  <div class="proof-card">
    <div class="proof-content">
      <div style="color: #ffb300; font-weight: bold; margin-bottom: 0.5rem">PROOF C</div>
      <h2>Everywhere</h2>
      <p>One primitive, consistent across your entire stack. The same engine that optimizes your cloud backend runs in your browser via WASM and on your edge devices.</p>
      <p>Write your optimization logic once in Rust or Python, deploy it anywhere.</p>
    </div>
    <div class="proof-visual">
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; width: 100%">
        <div style="background: #1e293b; padding: 1rem; border-radius: 8px; text-align: center">Server (Rust)</div>
        <div style="background: #1e293b; padding: 1rem; border-radius: 8px; text-align: center">Browser (WASM)</div>
        <div style="background: #1e293b; padding: 1rem; border-radius: 8px; text-align: center">Desktop (Python)</div>
        <div style="background: #1e293b; padding: 1rem; border-radius: 8px; text-align: center">Edge (Embedded)</div>
      </div>
    </div>
  </div>
</div>

## Use Cases

<div class="grid-4" style="margin-top: 2rem">
  <div>
    <h4>Serving Knobs</h4>
    <p style="font-size: 0.9rem; color: #94a3b8">Tune concurrency limits, timeouts, and batch sizes in real-time based on load.</p>
  </div>
  <div>
    <h4>Cache Policy</h4>
    <p style="font-size: 0.9rem; color: #94a3b8">Dynamically adjust TTL and eviction thresholds to maximize hit rates.</p>
  </div>
  <div>
    <h4>Kernel Selection</h4>
    <p style="font-size: 0.9rem; color: #94a3b8">Select the optimal compute kernel for the current matrix size and hardware state.</p>
  </div>
  <div>
    <h4>DB Tuning</h4>
    <p style="font-size: 0.9rem; color: #94a3b8">Adjust vacuum intensity and buffer pool usage to match query patterns.</p>
  </div>
</div>
