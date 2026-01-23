# Error Codes Reference

This page documents all error codes, exceptions, and common error messages in ArqonHPO.

---

## CLI Exit Codes

| Code  | Name            | Description                        |
| ----- | --------------- | ---------------------------------- |
| `0`   | Success         | Command completed successfully     |
| `1`   | GeneralError    | Unspecified error (check message)  |
| `2`   | ConfigError     | Config validation failed           |
| `3`   | StateError      | State file corrupt or incompatible |
| `10`  | BudgetExhausted | Optimization budget exhausted      |
| `20`  | ScriptError     | Evaluation script failed           |
| `130` | Interrupted     | Process interrupted (Ctrl+C)       |

**Example handling:**

```bash
arqonhpo run --config config.json --script ./eval.sh
case $? in
  0) echo "Optimization complete" ;;
  2) echo "Fix your config file" ;;
  10) echo "Budget exhausted - optimization finished" ;;
  20) echo "Evaluation script failed" ;;
  *) echo "Error: $?" ;;
esac
```

---

## Python Exceptions

All ArqonHPO Python exceptions inherit from `ArqonError`.

| Exception      | When Raised         | Example                          |
| -------------- | ------------------- | -------------------------------- |
| `ValueError`   | Invalid config JSON | Missing required field           |
| `RuntimeError` | Internal Rust panic | Unexpected state                 |
| `TypeError`    | Wrong argument type | Non-string passed to constructor |

**Example handling:**

```python
from arqonhpo import ArqonSolver
import json

try:
    solver = ArqonSolver(json.dumps({"seed": 42}))  # Missing budget!
except ValueError as e:
    print(f"Config error: {e}")
    # Output: Config error: missing field `budget`
```

---

## Config Validation Errors

Errors returned by `arqonhpo validate` or when constructing `ArqonSolver`:

| Error Message                   | Cause                   | Fix                                |
| ------------------------------- | ----------------------- | ---------------------------------- |
| `missing field 'seed'`          | Required field missing  | Add `"seed": 42`                   |
| `missing field 'budget'`        | Required field missing  | Add `"budget": 100`                |
| `missing field 'bounds'`        | Required field missing  | Add bounds object                  |
| `bounds.X.min >= bounds.X.max`  | Invalid range           | Ensure min < max                   |
| `invalid scale 'Foo'`           | Unknown scale type      | Use `Linear`, `Log`, or `Periodic` |
| `budget must be > 0`            | Zero or negative budget | Use positive integer               |
| `seed must be u64`              | Seed too large/negative | Use 0 to 2^64-1                    |
| `probe_ratio must be in [0, 1]` | Invalid ratio           | Use value 0.0-1.0                  |

---

## Tell/Seed Errors

Errors when calling `tell()` or `seed()`:

| Error Message            | Cause                    | Fix                      |
| ------------------------ | ------------------------ | ------------------------ |
| `invalid JSON`           | Malformed JSON string    | Validate JSON syntax     |
| `expected array`         | Root is not array        | Wrap results in `[...]`  |
| `missing field 'params'` | Result missing params    | Add params object        |
| `missing field 'value'`  | Result missing objective | Add value field          |
| `param X not in bounds`  | Unknown parameter        | Check param names        |
| `duplicate eval_id N`    | IDs not unique           | Use unique IDs in tell() |

---

## Rust Error Types

For Rust users, errors are returned as `Result<T, Error>`:

### Core Crate

```rust
pub enum SolverError {
    /// Config validation failed
    ConfigError(String),

    /// State is corrupt
    StateError(String),

    /// Budget exhausted
    BudgetExhausted,

    /// Invalid results
    ResultError(String),
}
```

### Hotpath Crate

```rust
pub enum Violation {
    DeltaTooLarge,
    RateLimitExceeded,
    DirectionFlipViolation,
    CumulativeDeltaViolation,
    InSafeMode,
    AuditQueueFull,
    NoBaseline,
}
```

See [Hotpath API Reference](hotpath.md) for violation details.

---

## Dashboard API Errors

HTTP errors from the dashboard REST API:

| Status | Endpoint          | Cause                 |
| ------ | ----------------- | --------------------- |
| `404`  | Any               | Invalid endpoint path |
| `400`  | POST /api/actions | Invalid action JSON   |
| `500`  | Any               | Internal server error |

---

## Common Runtime Issues

### "No candidates returned"

**Problem:** `ask()` or `ask_one()` returns `None` immediately.

**Causes:**

1. Budget already exhausted
2. Config has 0 budget

**Solution:**

```python
print(f"Budget: {solver.get_history_len()}")
```

---

### "Import failed: incompatible version"

**Problem:** Artifact from different ArqonHPO version.

**Solution:** Re-export with current version or use migration guide.

---

### "State file locked"

**Problem:** Another process is using the state file.

**Solution:** Ensure only one process uses state file at a time.

---

## Next Steps

- [Troubleshooting](../troubleshooting.md) — Full troubleshooting guide
- [CLI Reference](cli.md) — Command documentation
- [Hotpath API](hotpath.md) — Violation handling
