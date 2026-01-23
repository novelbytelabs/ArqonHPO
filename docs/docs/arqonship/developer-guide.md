# ArqonShip Developer Guide

## Getting Started

### Prerequisites

- Rust 1.82+ (use `rustup update stable`)
- Python 3.10+ (for pytest parsing tests)
- SQLite 3.x
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/novelbytelabs/ArqonHPO.git
cd ArqonHPO

# Build the ship crate
cargo build -p ship --release

# The binary is at target/release/ship
# Optionally, alias it:
alias arqon='./target/release/ship'
```

### Running Tests

```bash
# Unit tests
cargo test -p ship

# Integration tests (requires initialized project)
cd /path/to/test-project
arqon init
arqon scan
cargo test -p ship --test integration
```

## Project Structure

```
crates/ship/
├── Cargo.toml           # Dependencies
├── src/
│   ├── main.rs          # CLI entrypoint (Clap)
│   ├── config.rs        # Configuration loading
│   ├── lib.rs           # Library exports for testing
│   ├── oracle/          # Codebase Oracle module
│   │   ├── mod.rs       # Module exports + scan_codebase()
│   │   ├── parser.rs    # Rust parser
│   │   ├── parser_py.rs # Python parser
│   │   ├── graph.rs     # Node extraction
│   │   ├── edges.rs     # Edge extraction
│   │   ├── store.rs     # SQLite DAO
│   │   ├── schema.rs    # SQL migrations
│   │   ├── embed.rs     # Embedding model
│   │   ├── vector_store.rs # LanceDB client
│   │   ├── query.rs     # Query engine
│   │   ├── hash.rs      # Content hashing
│   │   └── incremental.rs # Skip unchanged files
│   ├── heal/            # Self-Healing module
│   │   ├── mod.rs       # Module exports
│   │   ├── parser_rust.rs # Cargo test parser
│   │   ├── parser_py.rs # Pytest parser
│   │   ├── context.rs   # Context builder
│   │   ├── llm.rs       # LLM interface
│   │   ├── prompts.rs   # Prompt templates
│   │   ├── loop.rs      # Healing state machine
│   │   ├── apply.rs     # Fix application
│   │   ├── verify.rs    # Verification gate
│   │   └── audit.rs     # Audit logging
│   └── ship/            # Release module
│       ├── mod.rs       # Module exports
│       ├── checks.rs    # Constitution checks
│       ├── commits.rs   # Commit parser
│       ├── version.rs   # SemVer calculator
│       └── github.rs    # GitHub API
└── tests/
    ├── graph_test.rs    # Graph extraction tests
    └── vector_test.rs   # Vector store tests
```

## Extending ArqonShip

### Adding a New Language Parser

1. Add the tree-sitter grammar dependency:

```bash
cargo add -p ship tree-sitter-java
```

2. Create `crates/ship/src/oracle/parser_java.rs`:

```rust
use tree_sitter::{Parser, Tree, Language};
use anyhow::{Result, Context};

pub struct JavaParser {
    parser: Parser,
}

impl JavaParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        let language: Language = tree_sitter_java::LANGUAGE.into();
        parser.set_language(&language)
            .context("Error loading Java grammar")?;
        Ok(Self { parser })
    }

    pub fn parse(&mut self, code: &str) -> Option<Tree> {
        self.parser.parse(code, None)
    }
}
```

3. Update `oracle/mod.rs` to include the new parser.

4. Extend `GraphBuilder` to handle Java AST nodes.

### Adding a New LLM Backend

1. Implement the `LlmClient` trait in `heal/llm.rs`:

```rust
pub trait LlmClient {
    fn generate_fix(&mut self, prompt: &str) -> Result<String>;
}

// Example: OpenAI backend
pub struct OpenAiClient {
    api_key: String,
    model: String,
}

impl LlmClient for OpenAiClient {
    fn generate_fix(&mut self, prompt: &str) -> Result<String> {
        // HTTP POST to OpenAI API
        todo!()
    }
}
```

2. Update `HealingLoop` to accept any `LlmClient`.

### Adding New Constitution Checks

1. Add a method to `ConstitutionCheck` in `ship/checks.rs`:

```rust
pub fn check_security_scan(&self) -> Result<bool> {
    let output = Command::new("cargo")
        .args(["audit"])
        .current_dir(&self.root)
        .output()?;

    Ok(output.status.success())
}
```

2. Call it from `run_all()`.

## Testing Guidelines

### Unit Tests

Place in the same file with `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
    }
}
```

### Integration Tests

Place in `crates/ship/tests/`:

```rust
// tests/oracle_integration.rs
use ship::oracle::scan_codebase;
use tempfile::tempdir;

#[tokio::test]
async fn test_scan_empty_project() {
    let dir = tempdir().unwrap();
    let result = scan_codebase(dir.path()).await;
    assert!(result.is_ok());
}
```

### Snapshot Tests

Using `insta`:

```rust
use insta::assert_debug_snapshot;

#[test]
fn test_graph_extraction() {
    let nodes = extract_nodes("fn foo() {}");
    assert_debug_snapshot!(nodes);
}
```

## Debugging

### Logging

Add `tracing` for structured logging:

```bash
RUST_LOG=debug arqon scan
```

### SQLite Inspection

```bash
sqlite3 .arqon/graph.db
sqlite> .schema
sqlite> SELECT * FROM nodes LIMIT 10;
```

### LanceDB Inspection

```python
import lancedb
db = lancedb.connect(".arqon/vectors.lance")
table = db.open_table("code_vectors")
print(table.head())
```

## CLI Usage Examples

### Codebase Oracle

```bash
# Index the entire codebase
arqon index

# Incremental update (only changed files)
arqon index --incremental

# Query for code
arqon chat -q "find the healing loop implementation"
arqon chat -q "where is the version bumping logic"
```

### Self-Healing CI

```bash
# Generate test output JSON
cargo test --message-format=json > test_output.json

# Run healing with defaults
arqon heal --log-file test_output.json

# Verbose mode with max attempts
arqon heal --log-file test_output.json --verbose --max-attempts 5

# Check audit log
sqlite3 .arqon/heal_audit.db "SELECT * FROM healing_attempts ORDER BY timestamp DESC LIMIT 5"
```

### Governed Release Pipeline

```bash
# Preview release (dry run)
arqon ship --dry-run

# Create release PR
arqon ship

# With custom branches
arqon ship --head-branch feature-xyz --base-branch develop

# Skip pre-flight checks (for testing)
arqon ship --skip-checks --dry-run
```

## Running Benchmarks

```bash
# Run oracle benchmarks
cargo run -p ship --example oracle_bench

# Expected output:
# Rust Parsing:
#   Per parse: ~50 µs
#   Throughput: ~20,000 parses/sec
#
# Python Parsing:
#   Per parse: ~20 µs
#   Throughput: ~50,000 parses/sec
```
