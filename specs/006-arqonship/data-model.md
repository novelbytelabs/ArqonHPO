# Data Model: ArqonShip

**Phase 1 Output** | **Date**: 2025-12-19
**Spec**: [006-arqonship/spec.md](spec.md)

## 1. Graph Database (SQLite)
File: `~/.arqon/graph.db`

### Table: `nodes`
Represents code entities (Files, Functions, Structs, Modules).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PK, Auto | Internal ID |
| `path` | TEXT | Not Null, Index | Relative file path |
| `type` | TEXT | Not Null | `function`, `struct`, `module`, `impl` |
| `name` | TEXT | Not Null, Index | Entity name (e.g. `solve`) |
| `start_line` | INTEGER | Not Null | 0-indexed start line |
| `end_line` | INTEGER | Not Null | 0-indexed end line |
| `signature_hash`| TEXT | Not Null | Content hash for change detection |
| `docstring` | TEXT | Nullable | Extracted docstring |

### Table: `edges`
Represents relationships between nodes.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `source_id` | INTEGER | FK -> nodes.id | The caller / importer |
| `target_id` | INTEGER | FK -> nodes.id | The callee / imported |
| `type` | TEXT | Not Null | `calls`, `imports`, `defines`, `impls` |

## 2. Vector Index (LanceDB)
Path: `~/.arqon/vectors.lance`

### Table: `code_vectors`
Embeddings for nodes.

| Field | Type | Description |
|-------|------|-------------|
| `id` | Int64 | Matches `nodes.id` |
| `vector` | Vector(768) | MiniLM embedding vector |
| `text` | String | The code snippet or signature embedded |

## 3. Healing Audit Log (SQLite)
File: `~/.arqon/audit.db` (or shared in `graph.db`)

### Table: `healing_attempts`
Records every autonomous repair attempt (Constitution XVII.4).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `run_id` | TEXT | PK | UUID of the healing run |
| `timestamp` | TEXT | Not Null | ISO8601 UTC |
| `file_path` | TEXT | Not Null | Target file |
| `error_msg` | TEXT | Not Null | Parsed error |
| `prompt_hash` | TEXT | Not Null | Hash of full prompt used |
| `diff_hash` | TEXT | Not Null | Hash of applied patch |
| `outcome` | TEXT | Not Null | `success`, `compile_fail`, `test_fail` |

## 4. Configuration
File: `.arqon/config.toml`

```toml
[meta]
config_version = 1 # SemVer of this config schema

[oracle]
include_globs = ["src/**/*.rs", "src/**/*.py"]
exclude_globs = ["target/", "venv/"]
model_path = "~/.arqon/models/"

[heal]
max_attempts = 2
model_id = "deepseek-coder-1.3b-instruct"
enabled = true

[ship]
require_branches = ["main"]
version_scheme = "semver"
```
