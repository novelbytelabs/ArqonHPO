# ArqonShip CLI Reference

## Installation

```bash
# Build from source
cd ArqonHPO
cargo build -p ship --release

# Add to PATH (optional)
cp target/release/ship ~/.local/bin/arqon
# OR
alias arqon='./target/release/ship'
```

## Global Options

```
arqon [OPTIONS] <COMMAND>

Options:
  --config <PATH>    Path to config file [default: .arqon/config.toml]
  -h, --help         Print help
  -V, --version      Print version
```

## Commands

### `arqon init`

Initialize ArqonShip in the current repository.

```bash
arqon init
```

**Behavior:**

- Creates `.arqon/` directory
- Generates default `config.toml`
- If config exists, does nothing

**Exit Codes:**

- `0`: Success
- `1`: Error creating files

---

### `arqon scan`

Build the Codebase Oracle (parse files, build graph, generate embeddings).

```bash
arqon scan
```

**Behavior:**

1. Walks project directory (respects `.gitignore`)
2. Parses `.rs` and `.py` files via Tree-sitter
3. Extracts functions, structs, classes → SQLite graph
4. Generates embeddings → LanceDB vectors
5. Shows progress spinner

**Output:**

```
Scanning codebase at "/path/to/project"
Processing lib.rs
Processing main.rs
...
Scan complete.
```

**Exit Codes:**

- `0`: Success
- `1`: Parse or storage error

---

### `arqon chat`

Query the Codebase Oracle using natural language.

```bash
arqon chat --query "How does authentication work?"
arqon chat -q "optimizer implementation"
```

**Options:**
| Flag | Description |
|------|-------------|
| `-q, --query <TEXT>` | Search query (required) |
| `--cli` | CLI output mode (default) |

**Output:**

```
[src/auth/mod.rs] authenticate (Score: 0.89)
[src/auth/jwt.rs] verify_token (Score: 0.76)
```

**Exit Codes:**

- `0`: Results found
- `1`: Error or no results

---

### `arqon heal`

Autonomous self-healing for test failures.

```bash
arqon heal
arqon heal --log-file test-output.json --max-attempts 3
```

**Options:**
| Flag | Description |
|------|-------------|
| `--log-file <PATH>` | Cargo test JSON output file |
| `--max-attempts <N>` | Max repair attempts [default: 2] |

**Behavior:**

1. Parse test failures from log file
2. Build repair context from Oracle
3. Generate fix via LLM
4. Apply fix (whole-block replacement)
5. Verify (compile + lint + test)
6. Log attempt to audit database
7. Repeat up to max attempts

**Exit Codes:**

- `0`: All failures healed
- `1`: Some failures remain

> **Note:** LLM integration is currently stubbed. Full implementation requires model download.

---

### `arqon ship`

Create a governed release (SemVer + Changelog + PR).

```bash
arqon ship
arqon ship --dry-run
arqon ship --skip-checks
```

**Options:**
| Flag | Description |
|------|-------------|
| `--dry-run` | Preview without creating PR |
| `--skip-checks` | Skip pre-flight constitution checks |

**Behavior:**

1. Run pre-flight checks:
   - Clean git working directory
   - All tests pass
   - No untagged TODO items (optional)
2. Parse commits since last tag
3. Calculate next SemVer version
4. Generate changelog
5. Create GitHub PR (requires `GITHUB_TOKEN`)

**Example Output:**

```
Starting release pipeline...
Next version: v1.2.0

Changelog:
## v1.2.0

### Features
- feat: Add authentication module
- feat: Implement rate limiting

### Bug Fixes
- fix: Resolve connection timeout

[DRY RUN] Would create release PR
```

**Exit Codes:**

- `0`: Release PR created / dry-run complete
- `1`: Pre-flight check failed

---

## Exit Codes Summary

| Code | Meaning                     |
| ---- | --------------------------- |
| `0`  | Command succeeded           |
| `1`  | Command failed (see stderr) |

## Examples

### Full workflow

```bash
# Initialize project
arqon init

# Build index
arqon scan

# Query code
arqon chat -q "error handling"

# After test failure
cargo test --message-format=json > test.json
arqon heal --log-file test.json

# Create release
export GITHUB_TOKEN=ghp_xxx
arqon ship --dry-run
arqon ship
```
