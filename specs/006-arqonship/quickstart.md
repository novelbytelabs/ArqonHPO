# ArqonShip Quickstart

## 1. Installation

ArqonShip is distributed as a single static binary.

```bash
# From source (Dev)
cargo install --path crates/ship

# Check version
arqon --version
```

## 2. Initialization

Initialize ArqonShip in your repo (creates `.arqon/config.toml`):

```bash
cd my-repo
arqon init
```

## 3. The First Scan

Build the Codebase Oracle (Graph + Embeddings):

```bash
arqon scan
```

*Note: The first scan downloads the embedding model (~80MB) and may take 10-20s. Subsequent scans are incremental.*

## 4. Chat with your Code

Query the local oracle:

```bash
arqon chat --query "Where is the SPSA optimizer defined?"
```

## 5. Self-Healing Workflow

1.  Make a breaking change (e.g., delete a semicolon).
2.  Run the heal command:

```bash
# Will run tests, detect failure, and attempt fix
arqon heal
```

3.  Verify the fix and commit.
