# Quickstart: ArqonHPO Development

**Branch**: `002-two-use-cases`

## Prerequisites
- Rust 1.75+ (`rustup update stable`)
- Python 3.10+
- `maturin` (`pip install maturin`)

## Build & Test

### 1. Core Logic (Rust)
Pure unit tests for the solver state machine.
```bash
cargo test -p arqonhpo-core
```

### 2. Python Bindings
Build the wheel and run integration tests.
```bash
# In repo root
maturin develop
pytest bindings/python/tests/
```

### 3. Run a Trace
```python
import arqonhpo as arq

config = {
    "budget": 100,
    "seed": 42,
    "bounds": {"x": {"min": -5.0, "max": 5.0}}
}

def objective(params):
    return params["x"]**2

res = arq.run(config, objective)
print(f"Best: {res.best_value}")
```
