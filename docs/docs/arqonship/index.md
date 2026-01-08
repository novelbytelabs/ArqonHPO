# ArqonShip

> DevSecOps Automation System for Intelligent Codebase Understanding, Self-Healing CI, and Governed Releases

## What is ArqonShip?

ArqonShip is a Rust-based CLI tool that provides three core capabilities:

1. **🔍 Codebase Oracle** - Semantic code understanding via graph + vector duality
2. **🔧 Self-Healing CI** - Autonomous repair of test failures using local AI
3. **🚀 Governed Releases** - Constitution-compliant release pipeline

## Quick Start

```bash
# Build
cargo build -p ship --release
alias arqon='./target/release/ship'

# Initialize in your project
arqon init

# Scan your codebase
arqon scan

# Query your code
arqon chat -q "How does error handling work?"

# Create a release
arqon ship --dry-run
```

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](architecture.md) | System design and data flows |
| [CLI Reference](cli-reference.md) | Command documentation |
| [Configuration](configuration.md) | Config file options |
| [Developer Guide](developer-guide.md) | Contributing and extending |

## Key Features

### Codebase Oracle
- **Tree-sitter parsing** for Rust and Python
- **Graph storage** (SQLite) for code structure
- **Vector embeddings** (LanceDB + MiniLM) for semantic search
- **Hybrid queries** combining structural and semantic matching

### Self-Healing CI
- **Log parsing** for cargo test and pytest
- **Context-aware repairs** using Oracle data
- **Local LLM inference** (DeepSeek-Coder via Candle)
- **Verification gates** ensuring fixes compile and pass tests
- **Audit logging** for all healing attempts

### Governed Releases
- **Pre-flight checks** (clean git, passing tests)
- **Conventional commit parsing**
- **Automatic SemVer calculation**
- **Changelog generation**
- **GitHub PR creation**

## Constitution Alignment

ArqonShip adheres to ArqonHPO Constitution Sections XVI-XIX:

- **XVI**: Graph + Vector duality for codebase understanding
- **XVII**: Self-Healing governance (2-attempt max, audit logging)
- **XVIII**: CI/CD automation (pre-flight checks, SemVer)
- **XIX**: CLI contracts (exit codes, structured output)

## Requirements

- Rust 1.82+
- SQLite 3.x
- ~2GB disk space for AI models
