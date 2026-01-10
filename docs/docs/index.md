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
    background: radial-gradient(circle at 50% 40%, rgba(99, 102, 241, 0.35) 0%, transparent 50%);
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
    font-weight: 700 !important;
    color: #fff !important;
    text-align: center !important;
    max-width: 900px;
    margin: 0 auto 2rem !important;
    line-height: 1.3;
    text-shadow: 0 2px 10px rgba(0,0,0,0.5);
  }
  
  .hero-support {
    font-size: 1.5rem;
    color: #cbd5e1;
    margin-bottom: 2rem;
    font-family: system-ui, -apple-system, sans-serif;
    font-weight: 500;
    max-width: 900px;
    margin-left: auto;
    margin-right: auto;
    line-height: 1.4;
  }
  
  .hero-bullets {
    display: flex;
    justify-content: center;
    gap: 2rem;
    margin-bottom: 3rem;
    flex-wrap: wrap;
  }
  
  .hero-bullet {
    background: rgba(30, 41, 59, 0.5);
    padding: 0.8rem 1.5rem;
    border-radius: 8px;
    border: 1px solid rgba(148, 163, 184, 0.2);
    color: #e2e8f0;
    font-weight: 500;
    font-size: 1.1rem;
    backdrop-filter: blur(4px);
  }
  
  .hero-engine {
    font-size: 1.2rem;
    color: #94a3b8;
    margin-bottom: 4rem;
    font-family: 'Inter', sans-serif;
  }
  .hero-engine code {
    background: rgba(99, 102, 241, 0.25);
    color: #a5b4fc;
    padding: 0.5rem 1rem;
    border-radius: 8px;
    border: 1px solid rgba(99, 102, 241, 0.4);
    font-family: monospace; 
    font-weight: 700;
    
    /* Glowing Orb Effect */
    box-shadow: 0 0 60px rgba(99, 102, 241, 0.5), inset 0 0 20px rgba(99, 102, 241, 0.2);
    text-shadow: 0 0 10px rgba(99, 102, 241, 0.5);
    animation: pulse-glow 3s ease-in-out infinite;
  }

  @keyframes pulse-glow {
    0%, 100% {
      box-shadow: 0 0 30px rgba(99, 102, 241, 0.3), inset 0 0 10px rgba(99, 102, 241, 0.1);
      transform: scale(1);
    }
    50% {
      box-shadow: 0 0 80px rgba(99, 102, 241, 0.6), inset 0 0 30px rgba(99, 102, 241, 0.4);
      transform: scale(1.02);
    }
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
  .grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 3rem;
    margin: 4rem 0;
  }
  .card-simple {
    background: #1e293b;
    padding: 2.5rem;
    border-radius: 16px;
    border: 1px solid #334155;
  }
  .card-simple h3 { margin-top: 0; color: #fff; }
  .card-simple p { color: #94a3b8; line-height: 1.6; }

  /* Adoption Ladder */
  .ladder-step {
    border-left: 2px solid #6366f1;
    padding-left: 2rem;
    margin-bottom: 2rem;
    position: relative;
  }
  .ladder-step::before {
    content: "";
    position: absolute;
    left: -9px;
    top: 0;
    width: 16px;
    height: 16px;
    background: #0f172a;
    border: 2px solid #6366f1;
    border-radius: 50%;
  }
  .ladder-title { font-size: 1.2rem; font-weight: 700; color: #fff; margin-bottom: 0.5rem; }
  .ladder-desc { color: #94a3b8; }
  
  /* Feature Grid */
  .grid-4 {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
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
    transform: translateY(-5px); 
    border-color: #6366f1; 
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
  }
  .card-icon { font-size: 2.5rem; margin-bottom: 1.5rem; }
  .card h3 { margin-top: 0; font-size: 1.3rem; margin-bottom: 1rem; color: #f8fafc; font-weight: 700; }
  .card p { color: #94a3b8; line-height: 1.6; margin: 0; }

  /* Mobile Responsive Fixes */
  @media (max-width: 768px) {
    .hero-title { font-size: 3.5rem !important; }
    .hero-subtitle { font-size: 1.25rem !important; }
    .hero-section { padding: 6rem 0; }
    .grid-2 { grid-template-columns: 1fr; }
    .hero-bullets { flex-direction: column; gap: 1rem; align-items: center; }
  }
</style>

<div class="hero-section">
  <div class="hero-content">
    <h1 class="hero-title">Arqon Runtime Optimizer</h1>
    <h2 class="hero-subtitle">Optimization isn't a workflow anymore.<br>It's a control loop.</h2>
    <p class="hero-support">Safe self-optimization, robustness, and resilience for live systems—with microsecond-class overhead and deterministic governance.</p>
    
    <div class="hero-bullets">
        <div class="hero-bullet">🛡️ Safe by Construction</div>
        <div class="hero-bullet">🎯 Deterministic & Replayable</div>
        <div class="hero-bullet">⚡ Hot-Path Ready</div>
    </div>
    
    <p class="hero-engine">Powered by <code>ArqonHPO</code></p>
    
    <div class="cta-buttons">
      <a href="documentation/quickstart/" class="btn btn-primary">Get Started</a>
      <a href="demos/" class="btn btn-secondary">See It Live</a>
    </div>
  </div>
</div>

<!-- 1. The Beachhead Picks (Fastest Adoption) -->
<div style="margin: 4rem 0;">
  <h2 style="text-align: center; font-size: 2.2rem; margin-bottom: 2rem;">Real-Time Policy Autopilot</h2>
  
  <!-- Poignant Example -->
  <div style="text-align: center; max-width: 700px; margin: 0 auto 3rem; background: rgba(15, 23, 42, 0.6); padding: 1.5rem; border-radius: 12px; border: 1px solid rgba(99, 102, 241, 0.2);">
    <div style="font-family: monospace; font-size: 1.25rem;">
        <span style="color: #94a3b8; text-decoration: line-through; opacity: 0.6;">const TIMEOUT = 300;</span>
        <span style="margin: 0 1rem; color: #6366f1;">→</span>
        <span style="color: #4ade80; font-weight: bold;">fn timeout(load, latency) -> ms</span>
    </div>
    <div style="margin-top: 0.8rem; font-size: 1rem; color: #cbd5e1;">
        Don't deploy a number. Deploy a <strong>policy</strong> that continuously finds the right number.
    </div>
  </div>
  <div class="grid-4">
    <div class="card">
      <div style="color: #6366f1; font-weight: 700; margin-bottom: 0.5rem; text-transform: uppercase; font-size: 0.85rem;">Beachhead 1</div>
      <h3 style="margin-top: 0;">Reliability Autopilot</h3>
      <p><strong>The Pain:</strong> Incident fatigue, dependency flaps, p99 spikes.</p><br />
      <p><strong>The Fix:</strong> Dynamically tune timeouts, retries, circuit breakers, and load shedding thresholds in response to telemetry.</p>
    </div>
    <div class="card">
      <div style="color: #6366f1; font-weight: 700; margin-bottom: 0.5rem; text-transform: uppercase; font-size: 0.85rem;">Beachhead 2</div>
      <h3 style="margin-top: 0;">Cache & Queue Control</h3>
      <p><strong>The Pain:</strong> Constant traffic drift makes static tuning impossible.</p><br />
      <p><strong>The Fix:</strong> Continuous adjustment of TTLs, admission policies, batch sizes, and queue limits to maximize throughput.</p>
    </div>
    <div class="card">
      <div style="color: #6366f1; font-weight: 700; margin-bottom: 0.5rem; text-transform: uppercase; font-size: 0.85rem;">Beachhead 3</div>
      <h3 style="margin-top: 0;">LLM / Inference Serving</h3>
      <p><strong>The Pain:</strong> Massive serving costs, unpredictable model mix.</p><br />
      <p><strong>The Fix:</strong> Autopilot for KV cache, spec decoding thresholds, and batching parameters. High Buyer Urgency.</p>
    </div>
    <div class="card" style="border: 1px dashed #475569; background: transparent;">
      <div style="color: #94a3b8; font-weight: 700; margin-bottom: 0.5rem; text-transform: uppercase; font-size: 0.85rem;">Expansion</div>
      <h3 style="margin-top: 0; color: #cbd5e1;">Also works for...</h3>
      <ul style="padding-left: 1.2rem; color: #94a3b8; line-height: 1.6; font-size: 0.9rem;">
          <li>Database Vacuuming</li>
          <li>Kernel Selection</li>
          <li>Mesh Routing</li>
          <li>Autoscaling Triggers</li>
      </ul>
    </div>
  </div>
</div>

<!-- 2. Category Definition -->
<div class="grid-2">
  <div class="card-simple">
    <div style="color: #94a3b8; font-weight: 700; margin-bottom: 1rem; text-transform: uppercase; font-size: 0.9rem;">Old World: Workflow</div>
    <p>You run experiments, wait, and manually retune. The system drifts between "tuning sessions."</p>
    <p style="color: #ef4444; font-weight: 600;">❌ Slow • Brittle • Human-bound</p>
  </div>
  <div class="card-simple" style="border-color: #6366f1; background: rgba(30, 41, 59, 0.4);">
    <div style="color: #818cf8; font-weight: 700; margin-bottom: 1rem; text-transform: uppercase; font-size: 0.9rem;">New World: Control Primitive</div>
    <p>The system continuously corrects itself <strong>inside</strong> the loop. Detects drift and applies bounded adjustments instantly.</p>
    <p style="color: #22c55e; font-weight: 600;">✅ Safe • Continuous • Auditable</p>
  </div>
</div>

<div style="text-align: center; max-width: 800px; margin: 4rem auto; padding: 2rem; background: #1e293b; border-radius: 12px; border: 1px solid #334155;">
    <h3 style="margin-top: 0; font-size: 1.5rem; color: #fff;">The Promise: Near-zero overhead, with safety guarantees.</h3>
    <p style="color: #cbd5e1; font-size: 1.1rem; line-height: 1.6;">ArqonHPO makes self-optimization cheap enough to run continuously—so resilience becomes a default property, not an ops project.</p>
</div>

<!-- 3. The 4 Proofs -->
<div style="margin: 6rem 0;">
    <h2 style="text-align: center; font-size: 2.5rem; margin-bottom: 3rem;">4 Proofs of the New Paradigm</h2>
    <div class="grid-4">
        <div class="card">
            <div style="color: #f59e0b; font-weight: 700; font-size: 0.9rem; margin-bottom: 0.5rem;">PROOF A: DRFT</div>
            <h3>Survive the Drift</h3>
            <p>Traffic shifts. Hardware ages. ArqonHPO acts as a <strong>homeostatic system</strong>, adapting in real-time to maintain optimal SLAs.</p>
        </div>
        <div class="card">
            <div style="color: #f59e0b; font-weight: 700; font-size: 0.9rem; margin-bottom: 0.5rem;">PROOF B: JITTER</div>
            <h3>Flatten the Jitter</h3>
            <p>Zero GC pauses. Engineered for the hot path. We prove <strong>sub-microsecond p99 overhead</strong> for every decision.</p>
        </div>
        <div class="card">
            <div style="color: #f59e0b; font-weight: 700; font-size: 0.9rem; margin-bottom: 0.5rem;">PROOF C: ANYWHERE</div>
            <h3>Everywhere</h3>
            <p>One primitive. Rust server, Python script, Browser WASM, Edge device. Same API, same safety guarantees.</p>
        </div>
        <div class="card" style="border-color: #6366f1;">
            <div style="color: #818cf8; font-weight: 700; font-size: 0.9rem; margin-bottom: 0.5rem;">PROOF D: ORGANISM</div>
            <h3>Organism Capabilities</h3>
            <p>Apply to every knob. The system evolves from a machine into an <strong>organism</strong> with its own self-regulating metabolism.</p>
        </div>
    </div>
    <div style="text-align: center; margin-top: 2rem;">
        <a href="benchmarks/" style="color: #818cf8; text-decoration: none; font-weight: 600;">View Full Benchmarks →</a>
    </div>
</div>

<!-- 4. Safety & Governance -->
<style>
  .governance-box {
    margin: 8rem 0;
    background: rgba(15, 23, 42, 0.4);
    padding: 5rem 3rem;
    border-radius: 24px;
    border: 1px solid rgba(99, 102, 241, 0.2);
    box-shadow: 0 0 50px rgba(0,0,0,0.3), inset 0 0 20px rgba(99, 102, 241, 0.05);
    position: relative;
    overflow: hidden;
  }
  .governance-box::before {
    content: "";
    position: absolute;
    top: 0; left: 0; right: 0; height: 1px;
    background: linear-gradient(90deg, transparent, rgba(99, 102, 241, 0.5), transparent);
  }
  .governance-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4rem;
    margin-top: 4rem;
  }
  .gov-item {
    position: relative;
    padding-left: 4rem;
  }
  .gov-number {
    position: absolute;
    left: 0;
    top: 0;
    font-size: 2.5rem;
    font-weight: 900;
    color: rgba(99, 102, 241, 0.15);
    line-height: 1;
  }
  .gov-item h4 {
    color: #fff;
    font-size: 1.25rem;
    margin: 0 0 0.75rem 0;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .gov-item p {
    color: #94a3b8;
    line-height: 1.6;
    margin: 0;
    font-size: 1rem;
  }

  /* Adoption Ladder Enhancements */
  .ladder-container {
    max-width: 900px;
    margin: 10rem auto;
  }
  .ladder-step {
    background: rgba(30, 41, 59, 0.3);
    border: 1px solid rgba(148, 163, 184, 0.1);
    border-radius: 12px;
    padding: 2rem 2.5rem;
    margin-bottom: 2rem;
    transition: all 0.3s ease;
    border-left: 4px solid #6366f1;
  }
  .ladder-step:hover {
    background: rgba(30, 41, 59, 0.5);
    border-color: rgba(99, 102, 241, 0.3);
    transform: translateX(10px);
  }
  .ladder-title {
    font-size: 1.3rem;
    font-weight: 800;
    color: #fff;
    margin-bottom: 0.5rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .ladder-phase {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 2px;
    color: #818cf8;
    background: rgba(99, 102, 241, 0.1);
    padding: 0.2rem 0.6rem;
    border-radius: 4px;
  }

  @media (max-width: 768px) {
    .governance-grid { grid-template-columns: 1fr; gap: 2.5rem; }
    .governance-box { padding: 3rem 1.5rem; }
  }
</style>

<div class="governance-box">
    <div style="text-align: center;">
        <h2 style="font-size: 2.8rem; font-weight: 800; margin-bottom: 1rem; color: #fff; letter-spacing: -1px;">Autonomy without Chaos</h2>
        <p style="color: #94a3b8; max-width: 600px; margin: 0 auto; font-size: 1.1rem;">Every action is bounded, attributable, and reversible. Trust is our primary product.</p>
    </div>
    
    <div class="governance-grid">
        <div class="gov-item">
            <span class="gov-number">01</span>
            <h4>🛡️ Allowlist Only</h4>
            <p>Unknown knobs are rejected. You explicitly define exactly which policies and parameters the control loop can touch.</p>
        </div>
        <div class="gov-item">
            <span class="gov-number">02</span>
            <h4>🛑 Bounded Deltas</h4>
            <p>Strict step-size limits and global bounds prevent wild oscillations or dangerous state transitions.</p>
        </div>
        <div class="gov-item">
            <span class="gov-number">03</span>
            <h4>⏪ Instant Rollback</h4>
            <p>Baseline restoration is an atomic, first-class operation. One call returns the system to a known-safe safety state.</p>
        </div>
        <div class="gov-item">
            <span class="gov-number">04</span>
            <h4>📜 Audit Trail</h4>
            <p>Every proposal and decision is immortalized in a lock-free event stream. You know exactly <em>why</em> the system changed.</p>
        </div>
    </div>
</div>

<!-- 5. Adoption Ladder -->
<div class="ladder-container">
    <h2 style="text-align: center; font-size: 2.5rem; margin-bottom: 4rem; font-weight: 800;">The Adoption Ladder</h2>
    
    <div class="ladder-step">
        <div class="ladder-title">
            Shadow Mode
            <span class="ladder-phase">Phase 01</span>
        </div>
        <div class="ladder-desc">Arqon reads telemetry and emits proposals, but does not apply them. Verify the autonomous logic against your team's manual decisions.</div>
    </div>
    
    <div class="ladder-step">
        <div class="ladder-title">
            Low-Risk Actuation
            <span class="ladder-phase">Phase 02</span>
        </div>
        <div class="ladder-desc">Enable actuation on reversible, low-blast-radius knobs like cache TTLs or batch sizes. Build confidence in the control layer.</div>
    </div>
    
    <div class="ladder-step">
        <div class="ladder-title">
            High Leverage Policies
            <span class="ladder-phase">Phase 03</span>
        </div>
        <div class="ladder-desc">Expand to timeouts, retries, and load-shedding thresholds. Allow Arqon to steer your system's reliability through volatility.</div>
    </div>
    
    <div class="ladder-step">
        <div class="ladder-title">
            Fleet Governance
            <span class="ladder-phase">Phase 04</span>
        </div>
        <div class="ladder-desc">Run local control loops on every node with centralized policy oversight. Autonomous resilience becomes a default property of your software.</div>
    </div>
</div>
