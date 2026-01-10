# Installation

This guide covers all installation methods for ArqonHPO.

---

## Python (Recommended)

The easiest way to get started:

```bash
pip install arqonhpo
```

### Requirements
- Python 3.10+ 
- 64-bit OS (Linux, macOS, Windows)

### Verify Installation
```bash
python -c "from arqonhpo import ArqonSolver; print('OK')"
```

---

## From Source (Rust)

For development or custom builds:

### Prerequisites
- Rust 1.82+ (`rustup update stable`)
- Protobuf compiler (`apt install protobuf-compiler` or `brew install protobuf`)

### Build
```bash
git clone https://github.com/novelbytelabs/ArqonHPO.git
cd ArqonHPO

# Build all crates
cargo build --release

# Install CLI
cargo install --path crates/cli --bin arqonhpo-cli
```

### Verify
```bash
arqonhpo-cli --version
```

---

## Python Bindings (Maturin)

To build Python bindings from source:

### Prerequisites
- Rust 1.82+
- Python 3.10+ with venv
- maturin (`pip install maturin`)

### Build
```bash
cd bindings/python
maturin develop --release
```

### Verify
```bash
python -c "from arqonhpo import ArqonSolver, ArqonProbe; print('OK')"
```

---

## Docker

Coming in v0.4. Track progress: [Issue #XX](https://github.com/novelbytelabs/ArqonHPO/issues)

---

## Platform Support

| Platform | Python | CLI | Status |
|----------|--------|-----|--------|
| Linux x86_64 | ✅ | ✅ | Full support |
| macOS x86_64 | ✅ | ✅ | Full support |
| macOS ARM64 | ✅ | ✅ | Full support |
| Windows x86_64 | ✅ | ✅ | Full support |
| Linux ARM64 | ✅ | ✅ | Full support |

---

## Troubleshooting

### `pip install` fails with "no matching distribution"
- Check Python version: `python --version` (need 3.10+)
- Check architecture: `python -c "import platform; print(platform.machine())"`

### Rust build fails with protobuf error
- Install protobuf: `apt install protobuf-compiler` 
- Or: `brew install protobuf`

### Import error: "symbol not found"
- Rebuild with matching Python version
- Use virtual environment to isolate

See [Troubleshooting](troubleshooting.md) for more issues.

---

## Next Steps

- [Quickstart](quickstart.md) — Get running in 5 minutes
- [Python API](reference/python.md) — API reference
