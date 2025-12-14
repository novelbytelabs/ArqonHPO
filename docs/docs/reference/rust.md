# Rust API Reference

Full API documentation is available via `cargo doc`.

```bash
cd ArqonHPO
cargo doc --open
```

## Quick Reference

### `arqonhpo_core::machine::Solver`

The core state machine with probe-classify-refine pipeline.

```rust
use arqonhpo_core::machine::Solver;
use arqonhpo_core::config::SolverConfig;

let config: SolverConfig = serde_json::from_str(r#"..."#)?;

// MVP mode (VarianceClassifier, UniformProbe)
let mut solver = Solver::new(config.clone());

// RPZL Production mode (ResidualDecayClassifier, PrimeIndexProbe, Top-K seeding)
let mut solver = Solver::rpzl(config);

loop {
    match solver.ask() {
        Some(candidates) => {
            // Evaluate candidates...
            solver.tell(results);
        }
        None => break,
    }
}
```

### `arqonhpo_core::classify::ResidualDecayClassifier`

RPZL algorithm classifier using α estimation from residual decay curves.

```rust
use arqonhpo_core::classify::{ResidualDecayClassifier, Classify, Landscape};

let classifier = ResidualDecayClassifier::default(); // α_threshold = 0.5

let (landscape, alpha) = classifier.classify(&history);
// α > 0.5 → Structured (use Nelder-Mead)
// α ≤ 0.5 → Chaotic (use TPE)
```

### `arqonhpo_core::probe::PrimeIndexProbe`

Multi-scale sampling using prime ratios for better structure detection.

```rust
use arqonhpo_core::probe::{PrimeIndexProbe, Probe};

let probe = PrimeIndexProbe::default();
let candidates = probe.sample(&config);
```

### `arqonhpo_core::strategies::tpe::BandwidthRule`

Adaptive bandwidth calculation for TPE kernel density estimation.

```rust
use arqonhpo_core::strategies::tpe::{TPE, BandwidthRule};

let tpe = TPE::with_bandwidth_rule(dim, BandwidthRule::Scott);
// Scott's Rule: σ = 1.06 × stddev × n^(-1/5)
```

### `arqonhpo_core::config::SolverConfig`

```rust
pub struct SolverConfig {
    pub seed: u64,
    pub budget: u64,
    pub bounds: HashMap<String, Domain>,
    pub probe_ratio: f64,
    pub strategy_params: Option<HashMap<String, f64>>,
}
```

### `arqonhpo_core::artifact::EvalTrace`

```rust
pub struct EvalTrace {
    pub eval_id: u64,
    pub params: HashMap<String, f64>,
    pub value: f64,
    pub cost: f64,
}
```

