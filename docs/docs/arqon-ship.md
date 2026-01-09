# ArqonShip User Guide

ArqonShip is a DevSecOps automation system providing intelligent codebase understanding, self-healing CI, and governed releases.

## Installation

```bash
# Build from source
cargo build -p ship --release

# Add to PATH
export PATH="$PWD/target/release:$PATH"
```

## Quick Start

### 1. Initialize ArqonShip

```bash
cd your-project
arqon init
```

This creates `.arqon/config.toml` with default settings.

### 2. Scan Your Codebase

```bash
arqon scan
```

Builds the Codebase Oracle (Graph + Vector index) for intelligent code understanding.

### 3. Query Your Code

```bash
arqon chat --query "How does the optimizer work?"
```

Semantic search across your codebase using the built index.

## Commands

### `arqon init`
Initialize ArqonShip in the current repository.

### `arqon scan`
Parse the codebase and build the graph/vector index.

### `arqon chat`
Query the codebase using natural language.

Options:
- `--query, -q`: The search query
- `--cli`: Use CLI output (default)

### `arqon heal`
Autonomous self-healing for failed tests.

Options:
- `--log-file`: Path to test output (JSON format)
- `--max-attempts`: Maximum repair attempts (default: 2)

### `arqon ship`
Create a governed release (SemVer + Changelog + PR).

Options:
- `--skip-checks`: Skip pre-flight constitution checks
- `--dry-run`: Preview without creating PR

## Configuration

Edit `.arqon/config.toml`:

```toml
[meta]
config_version = 1

[oracle]
include_globs = ["src/**/*.rs", "src/**/*.py"]
exclude_globs = ["target/", "venv/", ".git/"]
model_path = "~/.arqon/models/"

[heal]
max_attempts = 2
model_id = "deepseek-coder-1.3b-instruct"
enabled = true

[ship]
require_branches = ["main"]
version_scheme = "semver"
```

## Constitution Alignment

ArqonShip adheres to ArqonHPO Constitution Sections XVI-XIX:
- **XVI**: Codebase Oracle principles (Graph + Vector duality)
- **XVII**: Self-Healing CI governance (2 attempt max, audit logging)
- **XVIII**: CI/CD Automation (Pre-flight checks, SemVer)
- **XIX**: CLI Contracts (Exit codes, structured output)
