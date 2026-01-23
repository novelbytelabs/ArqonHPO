# Troubleshooting

Common issues and how to fix them.

---

## Installation Issues

### `pip install arqonhpo` fails

**"No matching distribution found"**

```
ERROR: Could not find a version that satisfies the requirement arqonhpo
```

**Fix:** Check Python version and platform:

```bash
python --version  # Need 3.10+
python -c "import platform; print(platform.machine())"  # Need x86_64 or arm64
```

---

### Import fails with symbol error

**"undefined symbol" or "symbol not found"**

This happens when the wheel was built with a different Python version.

**Fix:**

```bash
pip uninstall arqonhpo
pip install --no-cache-dir arqonhpo
```

Or use a virtual environment:

```bash
python -m venv .venv
source .venv/bin/activate
pip install arqonhpo
```

---

## Runtime Issues

### "Invalid config" error

```python
PyValueError: Invalid config: missing field `budget`
```

**Fix:** Ensure all required fields are present:

```python
config = {
    "seed": 42,       # Required
    "budget": 100,    # Required
    "bounds": {...}   # Required
}
```

---

### No candidates returned from `ask()`

**Problem:** `solver.ask()` returns `None` immediately.

**Causes:**

1. Budget already exhausted
2. Config has 0 budget
3. State file corrupted

**Fix:** Check budget and history:

```python
print(f"History: {solver.get_history_len()}")
# Ensure history_len < budget
```

---

### Performance is slow

**Problem:** Each `ask()` takes >100ms.

**Causes:**

1. Python object conversion overhead
2. Large parameter space
3. TPE with many history points

**Fix:**

- Use batch mode: `solver.ask()` returns multiple candidates
- Reduce history size in config
- Use Nelder-Mead for smooth functions

---

## CLI Issues

### "command not found: arqonhpo"

**Fix:** Install the CLI binary:

```bash
cargo install --path crates/cli --bin arqonhpo-cli
```

Or use the Python CLI:

```bash
python -m arqonhpo --help
```

---

### TUI shows blank screen

**Problem:** TUI starts but shows no data.

**Causes:**

1. State file doesn't exist or is empty
2. State file has incompatible format

**Fix:**

```bash
# Check state file exists
ls -la state.json

# Run an optimization first
arqonhpo ask --config config.json --state state.json
```

---

### Dashboard API returns 404

**Problem:** `curl http://localhost:3030/api/state` returns 404.

**Causes:**

1. Wrong URL (no /api prefix)
2. Dashboard not running

**Fix:**

```bash
# Correct endpoints
curl http://localhost:3030/api/state
curl http://localhost:3030/api/summary

# Verify dashboard is running
arqonhpo dashboard --state state.json --addr 127.0.0.1:3030
```

---

## Build Issues

### Protobuf compiler not found

```
error: failed to run custom build command for `arqonhpo-ffi`
```

**Fix:**

```bash
# Ubuntu/Debian
sudo apt install protobuf-compiler

# macOS
brew install protobuf

# Verify
protoc --version
```

---

### Rust version too old

```
error: package requires rustc 1.82.0 or newer
```

**Fix:**

```bash
rustup update stable
rustc --version  # Should be 1.82+
```

---

## Still Stuck?

1. Check [GitHub Issues](https://github.com/novelbytelabs/ArqonHPO/issues)
2. Open a [Discussion](https://github.com/novelbytelabs/ArqonHPO/discussions)
3. Review the [Constitution](../project/constitution.md) for design rationale
