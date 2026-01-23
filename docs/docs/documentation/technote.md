# ArqonHPO Technical Reference

> **pip install arqonhpo** — Microsecond-budget hyperparameter optimization.

## Overview

ArqonHPO is a Rust-cored, Python-wrapped hyperparameter optimization library that automatically adapts to your problem landscape using the **PCR (Probe-Classify-Refine)** algorithm.

**Key Features:**
- 🦀 **Rust Core**: Zero-allocation hot-path, O(log n) operations
- 🐍 **Python API**: Simple `ask() / tell()` interface
- 🔬 **Auto-Adaptive**: Detects smooth vs noisy landscapes
- 🎯 **Deterministic**: Same seed = same results
- ⚡ **Real-time Ready**: `ask_one()` for online control

## Installation

```bash
pip install arqonhpo
```

## Quick Reference

### ArqonSolver API

```python
from arqonhpo import ArqonSolver
import json

# Configuration
config = {
    "seed": 42,        # Reproducibility
    "budget": 100,     # Max evaluations
    "bounds": {
        "x": {"min": -10.0, "max": 10.0},
        "y": {"min": 0.001, "max": 1.0, "scale": "Log"}  # Log scale
    }
}

solver = ArqonSolver(json.dumps(config))
```

#### Methods

| Method | Purpose | Returns |
|:---|:---|:---|
| `ask()` | Get batch of candidates | `List[Dict[str, float]]` or `None` |
| `ask_one()` | Get single candidate (online mode) | `Dict[str, float]` or `None` |
| `tell(results_json)` | Report evaluation results | `None` |
| `seed(history_json)` | Warm-start with history | `None` |
| `get_history_len()` | Count of evaluations | `int` |

### ArqonProbe API

```python
from arqonhpo import ArqonProbe

probe = ArqonProbe(json.dumps(config), seed=42)
```

| Method | Purpose | Returns |
|:---|:---|:---|
| `sample_at(index)` | Deterministic sample at index | `Dict[str, float]` |
| `sample_range(start, count)` | Range of samples | `List[Dict[str, float]]` |

## Configuration Schema

```python
config = {
    # Required
    "seed": 42,                         # Random seed
    "budget": 100,                      # Max evaluations
    "bounds": {                         # Search space
        "param": {"min": 0.0, "max": 1.0},
        "log_param": {"min": 1e-5, "max": 1.0, "scale": "Log"},
        "angle": {"min": 0.0, "max": 360.0, "scale": "Periodic"}
    },
    
    # Optional
    "probe_ratio": 0.2,                 # Exploration fraction (default: 0.2)
    "strategy_params": {
        "gamma": 0.25,                  # TPE quantile threshold
        "n_startup": 10                 # Random samples before model
    }
}
```

## Usage Patterns

### Batch Mode (Full PCR)

```python
while True:
    batch = solver.ask()
    if batch is None:
        break
    
    results = []
    for i, params in enumerate(batch):
        value = objective(params)
        results.append({
            "eval_id": i,
            "params": params,
            "value": value,
            "cost": 1.0
        })
    
    solver.tell(json.dumps(results))
```

### Online Mode (Real-time)

```python
for step in range(1000):
    candidate = solver.ask_one()
    if candidate is None:
        break
    
    reward = apply_to_system(candidate)
    
    solver.seed(json.dumps([{
        "params": candidate,
        "value": -reward,  # Minimize
        "cost": 1.0
    }]))
```

### Distributed Mode (ArqonProbe)

```python
# Worker N samples indices [N*chunk, (N+1)*chunk)
worker_id = 5
chunk_size = 100
samples = probe.sample_range(worker_id * chunk_size, chunk_size)
```

## Result Schema

```python
{
    "eval_id": 0,           # Batch identifier
    "params": {"x": 1.5},   # Configuration evaluated
    "value": 0.42,          # Objective value (minimize)
    "cost": 1.0             # Relative runtime cost
}
```

## Algorithm Details

### PCR Pipeline

```
┌─────────┐    ┌──────────┐    ┌────────┐
│  PROBE  │ -> │ CLASSIFY │ -> │ REFINE │
│  (LDS)  │    │ (Smooth?) │    │(NM/TPE)│
└─────────┘    └──────────┘    └────────┘
```

1. **Probe**: Low-discrepancy sampling (Prime-Sqrt-Slopes-Rot)
2. **Classify**: Detect structured vs noisy landscape
3. **Refine**: Nelder-Mead (smooth) or TPE (noisy)

### Landscape Detection

| Classification | Algorithm Used | Typical Use Case |
|:---|:---|:---|
| **Structured** | Nelder-Mead | Simulation, physics |
| **Noisy** | TPE | ML training, stochastic |

## Performance

| Operation | Time | Notes |
|:---|:---|:---|
| `ask()` | ~1µs | Zero-allocation |
| `tell()` | ~100µs | Model update |
| `sample_at()` | ~50ns | Pure math |

## Version Info

```python
import arqonhpo
print(arqonhpo.__version__)  # 0.3.0
```

## Links

- [GitHub](https://github.com/novelbytelabs/ArqonHPO)
- [PyPI](https://pypi.org/project/arqonhpo/)
- [Full Documentation](https://novelbytelabs.github.io/ArqonHPO)
