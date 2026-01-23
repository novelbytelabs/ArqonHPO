# Rust API Reference

Full API documentation is available via `cargo doc`.

```bash
cd ArqonHPO
cargo doc --open
```

---

## Crate Overview

| Crate              | Description                               |
| ------------------ | ----------------------------------------- |
| `arqonhpo_core`    | High-level Solver, Strategies, Classifier |
| `arqonhpo_hotpath` | Low-level SafetyExecutor, SPSA, Telemetry |

---

## Core Crate (`arqonhpo_core`)

### `Solver`

The core state machine with probe-classify-refine pipeline.

```rust
use arqonhpo_core::machine::Solver;
use arqonhpo_core::config::SolverConfig;

let config: SolverConfig = serde_json::from_str(r#"{
    "seed": 42,
    "budget": 100,
    "bounds": {
        "x": {"min": -5.0, "max": 5.0, "scale": "Linear"},
        "y": {"min": 0.01, "max": 10.0, "scale": "Log"}
    }
}"#)?;

// Standard PCR mode (recommended)
let mut solver = Solver::pcr(config);

loop {
    match solver.ask() {
        Some(candidates) => {
            let results = evaluate_batch(&candidates);
            solver.tell(results);
        }
        None => break,
    }
}

println!("Best: {:?}", solver.best());
```

### Constructor Variants

| Method                | Mode           | Use Case                     |
| --------------------- | -------------- | ---------------------------- |
| `Solver::new(config)` | MVP mode       | Testing, simple problems     |
| `Solver::pcr(config)` | PCR production | Production use (recommended) |

---

### `SolverConfig`

```rust
pub struct SolverConfig {
    /// RNG seed for reproducibility
    pub seed: u64,

    /// Maximum evaluations
    pub budget: u64,

    /// Parameter bounds
    pub bounds: HashMap<String, Domain>,

    /// Fraction of budget for probe phase (0.0-1.0)
    pub probe_ratio: f64,

    /// Optional batch size override
    pub batch_size: Option<u64>,

    /// Force specific strategy (bypasses classifier)
    pub strategy: Option<StrategyType>,

    /// Strategy-specific parameters
    pub strategy_params: Option<HashMap<String, f64>>,
}
```

---

### `Domain`

Defines parameter bounds and scaling.

```rust
pub struct Domain {
    pub min: f64,
    pub max: f64,
    pub scale: Scale,
}

pub enum Scale {
    Linear,
    Log,
    Periodic { period: f64 },
}
```

**Example:**

```rust
use arqonhpo_core::config::{Domain, Scale};

let domain = Domain {
    min: 0.001,
    max: 1.0,
    scale: Scale::Log,
};
```

---

### `EvalTrace`

A single evaluation result.

```rust
pub struct EvalTrace {
    pub eval_id: u64,
    pub params: HashMap<String, f64>,
    pub value: f64,
    pub cost: f64,
}
```

---

## PCR Algorithm Components

### `ResidualDecayClassifier`

Classifies the landscape based on residual decay.

```rust
use arqonhpo_core::classify::{ResidualDecayClassifier, Classify, Landscape};

let classifier = ResidualDecayClassifier::new(0.5); // α threshold

let (landscape, alpha) = classifier.classify(&history);

match landscape {
    Landscape::Structured => println!("Use Nelder-Mead"),
    Landscape::Chaotic => println!("Use TPE"),
}
```

### `PrimeIndexProbe`

Low-discrepancy sequence sampling using prime ratios.

```rust
use arqonhpo_core::probe::{PrimeIndexProbe, Probe};

let probe = PrimeIndexProbe::new(seed);
let candidates = probe.sample(&config, batch_size);
```

---

## Strategies

### `NelderMead`

Simplex-based optimizer for structured landscapes.

```rust
use arqonhpo_core::strategies::nelder_mead::NelderMead;

let nm = NelderMead::new(
    dim,
    reflection_coeff,   // α = 1.0
    expansion_coeff,    // γ = 2.0
    contraction_coeff,  // ρ = 0.5
    shrink_coeff,       // σ = 0.5
);
```

### `MultiStartNM`

Multi-start Nelder-Mead for multimodal optimization.

```rust
use arqonhpo_core::strategies::multi_start_nm::MultiStartNM;

let msnm = MultiStartNM::new(dim, n_starts, seed);
```

### `TPE`

Tree-structured Parzen Estimator for noisy/chaotic landscapes.

```rust
use arqonhpo_core::strategies::tpe::{TPE, BandwidthRule};

let tpe = TPE::with_bandwidth_rule(dim, BandwidthRule::Scott);
// Scott's Rule: σ = 1.06 × stddev × n^(-1/5)
```

#### `BandwidthRule`

| Rule        | Formula                           | Use Case          |
| ----------- | --------------------------------- | ----------------- |
| `Scott`     | 1.06 × σ × n^(-1/5)               | General (default) |
| `Silverman` | 0.9 × min(σ, IQR/1.34) × n^(-1/5) | Outlier-robust    |
| `Fixed(bw)` | User-specified                    | Custom            |

---

## Seeding (Warm-Start)

### `SeedingConfig`

Configure how probe history seeds refiners.

```rust
use arqonhpo_core::seeding::{SeedingConfig, SeedingStrategy};

let config = SeedingConfig {
    strategy: SeedingStrategy::TopK { k: 10 },
    budget_fraction: 0.1,
};
```

### `SeedingStrategy`

| Strategy       | Description                  |
| -------------- | ---------------------------- |
| `TopK { k }`   | Top k best points from probe |
| `AllProbe`     | All probe history            |
| `Random { n }` | Random n points              |

---

## Hotpath Crate

For low-level safety-critical APIs (SafetyExecutor, SPSA, Telemetry), see:

→ **[Hotpath API Reference](hotpath.md)**

---

## Feature Flags

| Flag      | Description                     |
| --------- | ------------------------------- |
| `python`  | Enable Python bindings via PyO3 |
| `metrics` | Enable Prometheus metrics       |
| `tracing` | Enable tracing spans            |

```toml
[dependencies]
arqonhpo_core = { version = "0.3", features = ["metrics"] }
```

---

## Next Steps

- [Hotpath API](hotpath.md) — SafetyExecutor, SPSA, Guardrails
- [Python API](python.md) — Python bindings
- [Strategies](../concepts/strategies.md) — Algorithm comparison
