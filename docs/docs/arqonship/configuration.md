# ArqonShip Configuration Reference

## Configuration File

ArqonShip uses `.arqon/config.toml` for project-level settings.

### Generating Default Config

```bash
arqon init
```

Creates `.arqon/config.toml` with defaults.

## Full Configuration Schema

```toml
[meta]
# Configuration version (for migrations)
config_version = 1

[oracle]
# Glob patterns for files to include in scanning
include_globs = [
    "src/**/*.rs",
    "src/**/*.py",
    "lib/**/*.rs",
]

# Glob patterns for files/directories to exclude
exclude_globs = [
    "target/",
    "node_modules/",
    "venv/",
    ".git/",
    "*.min.js",
]

# Path to store/cache model files
model_path = "~/.arqon/models/"

[heal]
# Maximum repair attempts before giving up (Constitution XVII.1)
max_attempts = 2

# LLM model identifier for local inference
model_id = "deepseek-coder-1.3b-instruct"

# Enable/disable self-healing feature
enabled = true

[ship]
# Branches allowed for release (Constitution XVIII)
require_branches = ["main", "release/*"]

# Version scheme: "semver" or "calver"
version_scheme = "semver"
```

## Configuration Options

### `[meta]` Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `config_version` | integer | `1` | Schema version for future migrations |

### `[oracle]` Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `include_globs` | string[] | `["src/**/*.rs", "src/**/*.py"]` | Files to parse and index |
| `exclude_globs` | string[] | `["target/", "venv/", ".git/"]` | Paths to skip |
| `model_path` | string | `"~/.arqon/models/"` | Model cache directory |

### `[heal]` Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_attempts` | integer | `2` | Max healing iterations per failure |
| `model_id` | string | `"deepseek-coder-1.3b-instruct"` | Local LLM model |
| `enabled` | boolean | `true` | Feature toggle |

### `[ship]` Section

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `require_branches` | string[] | `["main"]` | Branches allowed for shipping |
| `version_scheme` | string | `"semver"` | Versioning strategy |

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `GITHUB_TOKEN` | GitHub API token for PR creation | For `arqon ship` |
| `ARQON_CONFIG` | Override config file path | No |
| `ARQON_MODEL_CACHE` | Override model cache directory | No |

## Data Directories

| Path | Purpose |
|------|---------|
| `.arqon/config.toml` | Project configuration |
| `.arqon/graph.db` | SQLite graph database |
| `.arqon/vectors.lance/` | LanceDB vector storage |
| `~/.arqon/models/` | Cached AI models |
| `~/.arqon/audit.db` | Global audit log |

## Example Configurations

### Minimal (Rust-only project)

```toml
[meta]
config_version = 1

[oracle]
include_globs = ["src/**/*.rs"]

[heal]
enabled = false  # Manual review only
```

### Full-stack project

```toml
[meta]
config_version = 1

[oracle]
include_globs = [
    "backend/**/*.rs",
    "frontend/**/*.ts",
    "scripts/**/*.py",
]
exclude_globs = [
    "target/",
    "node_modules/",
    "dist/",
    "*.test.ts",
]

[heal]
max_attempts = 3
enabled = true

[ship]
require_branches = ["main", "release/*"]
```
