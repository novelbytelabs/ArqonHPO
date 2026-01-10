# ADR-002: Python Bridge

**Status:** Accepted  
**Date:** 2024-12-13

## Context

Python is the dominant language for ML/Data Science. We need first-class Python support.

## Decision

Use **PyO3 with Maturin** for Python bindings.

### Binding Strategy

1. **JSON I/O:** Config and results are passed as JSON strings. This avoids complex type mappings and makes the API debuggable.

2. **Module Structure:**

   ```
   arqonhpo/
   ├── __init__.py          # Re-exports from _internal
   └── _internal.cpython-*.so  # Rust extension
   ```

3. **Thread Safety:** All traits (`Strategy`, `Probe`, `Classify`) require `Send + Sync` bounds to satisfy PyO3's GIL requirements.

### Why JSON?

- **Debugging:** Users can `print(config_json)` to inspect.
- **Serialization:** Easy to save/load configs.
- **Simplicity:** Avoid PyO3 type conversion complexity.

## Consequences

- **Pro:** Simple, debuggable API.
- **Pro:** No pyo3 type mapping bugs.
- **Con:** Slight overhead from JSON parsing (negligible vs. objective eval time).
