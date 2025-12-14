# Rust API Reference

Full API documentation is available via `cargo doc`.

```bash
cd ArqonHPO
cargo doc --open
```

## Quick Reference

### `arqonhpo_core::machine::Solver`

The core state machine.

```rust
use arqonhpo_core::machine::Solver;
use arqonhpo_core::config::SolverConfig;

let config: SolverConfig = serde_json::from_str(r#"..."#)?;
let mut solver = Solver::new(config);

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
