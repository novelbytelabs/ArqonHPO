<h1>Installation</h1>

<h3>Get ArqonHPO running on your system in minutes.</h3>

Whether you're optimizing your first hyperparameter or building production-grade ML pipelines, this guide has you covered.

---

## Choose Your Path

| Method                                                        | Best For                     | Time to Start |
| ------------------------------------------------------------- | ---------------------------- | ------------- |
| [**Python (pip)**](#python-recommended)                       | Most users, ML workflows     | 30 seconds    |
| [**Python (conda)**](#conda)                                  | Reproducible environments    | 1 minute      |
| [**Rust (from source)**](#from-source-rust)                   | Contributors, custom builds  | 5 minutes     |
| [**Python bindings (from source)**](#python-bindings-maturin) | Binding development          | 3 minutes     |
| [**Docker**](#docker)                                         | Isolated environments, CI/CD | 2 minutes     |

---

## Python (Recommended)

The fastest way to get started with ArqonHPO:

```bash
pip install arqonhpo
```

That's it. You're ready to optimize.

!!! note "No Rust Required"
The pip package includes pre-built binaries for all major platforms. You only need Rust if [building from source](#from-source-rust).

### Requirements

| Requirement | Version                         | How to Check       |
| ----------- | ------------------------------- | ------------------ |
| Python      | 3.10+                           | `python --version` |
| pip         | 21.0+                           | `pip --version`    |
| OS          | 64-bit Linux, macOS, or Windows | —                  |

### Verify Installation

```bash
python -c "from arqonhpo import ArqonSolver; print('✓ ArqonHPO installed successfully')"
```

### Upgrade to Latest

```bash
pip install --upgrade arqonhpo
```

### Install Specific Version

```bash
pip install arqonhpo==0.3.0
```

---

## Conda

For reproducible environments and scientific workflows:

```bash
conda install -c conda-forge arqonhpo
```

### Create a Dedicated Environment

```bash
# Create environment with ArqonHPO
conda create -n hpo python=3.11 arqonhpo -c conda-forge

# Activate it
conda activate hpo

# Verify
python -c "from arqonhpo import ArqonSolver; print('✓ Ready')"
```

### With Other ML Libraries

```bash
conda create -n ml-tuning python=3.11 arqonhpo pytorch scikit-learn -c conda-forge -c pytorch
```

---

## From Source (Rust)

Build ArqonHPO from source for development, customization, or to use the bleeding-edge version.

!!! warning "Rust 1.82 Required"
Building from source requires **exactly Rust 1.82**. This version is frozen due to specific language features and API dependencies used in the codebase.

### Prerequisites

Before you begin, ensure you have the following installed:

#### 1. Rust Toolchain (1.82 — Pinned)

```bash
# Install Rust via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update to latest stable
rustup update stable

# Verify version
rustc --version  # Must show exactly 1.82.x
```

??? tip "Windows Users"
On Windows, download and run [rustup-init.exe](https://rustup.rs/) instead. You may also need to install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).

#### 2. Protobuf Compiler

ArqonHPO uses Protocol Buffers for efficient serialization.

=== "Ubuntu/Debian"
`bash
    sudo apt update && sudo apt install -y protobuf-compiler
    `

=== "macOS"
`bash
    brew install protobuf
    `

=== "Fedora/RHEL"
`bash
    sudo dnf install -y protobuf-compiler
    `

=== "Arch Linux"
`bash
    sudo pacman -S protobuf
    `

=== "Windows"
```powershell # Using Chocolatey
choco install protoc

    # Or using Scoop
    scoop install protobuf
    ```

Verify protobuf installation:

```bash
protoc --version  # Should show libprotoc 3.x or higher
```

#### 3. Just (Task Runner)

We use [Just](https://github.com/casey/just) as our task runner for consistent developer experience.

=== "Cargo (Any Platform)"
`bash
    cargo install just
    `

=== "macOS"
`bash
    brew install just
    `

=== "Ubuntu/Debian"
`bash
    sudo apt install just
    # Or use cargo install just if not available
    `

=== "Windows"
```powershell # Using Chocolatey
choco install just

    # Using Scoop
    scoop install just
    ```

#### 4. Git

```bash
git --version  # Should show 2.x or higher
```

### Build Steps

```bash
# 1. Clone the repository
git clone https://github.com/novelbytelabs/ArqonHPO.git
cd ArqonHPO

# 2. Build all crates in release mode
cargo build --workspace --release

# 3. Run the test suite to verify everything works
cargo test --workspace
```

!!! success "Build Complete"
If all tests pass, you've successfully built ArqonHPO from source!

### Install the CLI

The ArqonHPO CLI provides interactive optimization via the terminal:

```bash
# Install the CLI binary
cargo install --path crates/cli --bin arqonhpo-cli

# Verify installation
arqonhpo-cli --version
```

The binary will be installed to `~/.cargo/bin/`. Ensure this is in your `PATH`.

### Development Build (Debug Mode)

For faster compilation during development:

```bash
cargo build --workspace
cargo test --workspace
```

---

## Python Bindings (Maturin)

Build Python bindings from source when contributing to the Python API or testing unreleased features.

### Prerequisites

- **Rust 1.82** (see [above](#1-rust-toolchain-182--pinned))
- **Python 3.10+** with a virtual environment
- **maturin** (Python-Rust build tool)

### Setup

```bash
# 1. Clone the repository
git clone https://github.com/novelbytelabs/ArqonHPO.git
cd ArqonHPO

# 2. Create a virtual environment
python -m venv venv
source venv/bin/activate  # Linux/macOS
# Or: venv\Scripts\activate  # Windows

# 3. Install maturin
pip install maturin

# 4. Build and install the bindings in development mode
cd bindings/python
maturin develop --release
```

### Verify

```bash
python -c "from arqonhpo import ArqonSolver, ArqonProbe; print('✓ Bindings installed')"
```

### Build Distributable Wheels

To create wheel files for distribution:

```bash
# Build wheels for your platform
maturin build --release -m bindings/python/Cargo.toml

# Wheels are output to target/wheels/
ls target/wheels/
```

---

## Docker

Containerized ArqonHPO for isolated environments and CI/CD pipelines.

### Quick Start

```bash
# Pull the official image
docker pull novelbytelabs/arqonhpo:latest

# Run interactively
docker run -it novelbytelabs/arqonhpo:latest python
>>> from arqonhpo import ArqonSolver
>>> print("Ready!")
```

### Mount Your Project

```bash
docker run -it -v $(pwd):/workspace novelbytelabs/arqonhpo:latest bash
```

### Docker Compose

For complex setups:

```yaml
# docker-compose.yml
version: "3.8"
services:
  optimizer:
    image: novelbytelabs/arqonhpo:latest
    volumes:
      - ./experiments:/workspace
    command: python /workspace/run_hpo.py
```

### Build Custom Image

```dockerfile
FROM novelbytelabs/arqonhpo:latest

# Add your dependencies
RUN pip install torch transformers

# Copy your code
COPY . /app
WORKDIR /app

CMD ["python", "optimize.py"]
```

!!! note "Docker Image Availability"
Docker images are published starting from v0.4. For earlier versions, build from source using the Dockerfile in the repository.

---

## Platform Support

### Officially Supported

| Platform                    | Python | CLI | Notes                                        |
| --------------------------- | :----: | :-: | -------------------------------------------- |
| Linux x86_64                |   ✅   | ✅  | Full support, primary development platform   |
| Linux ARM64                 |   ✅   | ✅  | Full support (AWS Graviton, Raspberry Pi 4+) |
| macOS x86_64                |   ✅   | ✅  | Full support                                 |
| macOS ARM64 (Apple Silicon) |   ✅   | ✅  | Full support, native M1/M2/M3 binaries       |
| Windows x86_64              |   ✅   | ✅  | Full support, tested on Windows 10/11        |

### Community Supported

These platforms may work but are not actively tested:

| Platform            | Status | Notes                                    |
| ------------------- | ------ | ---------------------------------------- |
| FreeBSD             | 🔧     | Build from source, may require patches   |
| Windows ARM64       | 🔧     | Untested, requires building from source  |
| musl Linux (Alpine) | 🔧     | Use `--target x86_64-unknown-linux-musl` |

---

## Virtual Environment Best Practices

We strongly recommend using virtual environments to isolate ArqonHPO from other projects.

### Option 1: venv (Built-in)

```bash
# Create environment
python -m venv arqon-env

# Activate
source arqon-env/bin/activate  # Linux/macOS
arqon-env\Scripts\activate     # Windows

# Install
pip install arqonhpo
```

### Option 2: conda

```bash
conda create -n arqon python=3.11
conda activate arqon
pip install arqonhpo
```

### Option 3: uv (Fast & Modern)

[uv](https://github.com/astral-sh/uv) is a blazing-fast Python package manager:

```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Create environment and install
uv venv
source .venv/bin/activate
uv pip install arqonhpo
```

---

## Troubleshooting

### Installation Issues

??? failure "`pip install` fails with 'no matching distribution'"
**Cause**: Your Python version or platform is not supported.

    **Solution**:
    ```bash
    # Check Python version (need 3.10+)
    python --version

    # Check architecture
    python -c "import platform; print(platform.machine())"
    ```

    If you're on an unsupported platform, [build from source](#from-source-rust).

??? failure "Rust build fails with protobuf error"
**Error**: `Could not find protoc installation`

    **Solution**: Install protobuf compiler for your platform:
    ```bash
    # Ubuntu/Debian
    sudo apt install protobuf-compiler

    # macOS
    brew install protobuf

    # Verify
    protoc --version
    ```

??? failure "Import error: 'symbol not found'"
**Cause**: Python version mismatch between build and runtime.

    **Solution**:
    ```bash
    # Ensure you're using the same Python that built the bindings
    which python

    # Rebuild in a clean environment
    pip uninstall arqonhpo
    pip install arqonhpo --no-cache-dir
    ```

??? failure "Permission denied during install"
**Solution**: Use a virtual environment instead of system Python:
`bash
    python -m venv venv
    source venv/bin/activate
    pip install arqonhpo
    `

??? failure "Windows: 'cargo' is not recognized"
**Cause**: Rust not in PATH after installation.

    **Solution**:
    1. Close and reopen your terminal
    2. Or manually add `%USERPROFILE%\.cargo\bin` to your PATH

### Runtime Issues

??? failure "Performance is slower than expected"
**Checklist**:

    - [ ] Using release build? (`cargo build --release`)
    - [ ] Not running under debugger?
    - [ ] CPU not throttled? (check power settings on laptops)
    - [ ] No other heavy processes competing for resources?

??? failure "Out of memory during optimization"
**Solutions**:

    1. Reduce population size in strategies
    2. Use streaming evaluation for large datasets
    3. Increase system swap space
    4. Consider using chunked optimization

### Getting More Help

If your issue isn't listed above:

1. **Search existing issues**: [GitHub Issues](https://github.com/novelbytelabs/ArqonHPO/issues)
2. **Ask in discussions**: [GitHub Discussions](https://github.com/novelbytelabs/ArqonHPO/discussions)
3. **Open a new issue** with:
   - Your OS and version
   - Python version
   - ArqonHPO version
   - Full error message/stack trace

---

## Uninstalling

### Python Package

```bash
pip uninstall arqonhpo
```

### Rust CLI

```bash
cargo uninstall arqonhpo-cli

# Or manually remove
rm ~/.cargo/bin/arqonhpo-cli
```

### Complete Cleanup

```bash
# Remove pip cache
pip cache purge

# Remove cargo artifacts (if built from source)
cd ArqonHPO && cargo clean
```

---

## Next Steps

You're all set! Here's where to go next:

<div class="grid cards" markdown>

- :rocket: **[Quickstart Guide](quickstart.md)**

  Get running in 5 minutes with your first optimization

- :book: **[Python API Reference](reference/python.md)**

  Complete API documentation

- :bulb: **[Examples & Tutorials](../examples/index.md)**

  Learn by example with real-world use cases

- :fontawesome-solid-people-group: **[Contributing](../project/CONTRIBUTING.md)**

  Help make ArqonHPO even better

</div>

---

**Having trouble?** [Open an issue](https://github.com/novelbytelabs/ArqonHPO/issues/new) or [ask in discussions](https://github.com/novelbytelabs/ArqonHPO/discussions).

**Found ArqonHPO useful?** Consider [starring the repo](https://github.com/novelbytelabs/ArqonHPO) :star:
