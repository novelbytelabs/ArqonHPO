# ArqonShip CLI Contract

**Version**: 1.0.0
**Spec**: [006-arqonship/spec.md](../spec.md)

## Global Flags
- `--config <PATH>`: Path to config file (default: `.arqon/config.toml`)
- `--verbose`: Enable debug logging
- `--offline`: Force offline mode (fail if network required)

## Commands

### 1. `arqon scan`
**Purpose**: Update the local codebase oracle (Graph + Vectors).
- **Flags**:
    - `--force`: Rebuild index from scratch.
    - `--verify`: Verify index consistency (hash check).
- **Exit Codes**:
    - 0: Success
    - 1: Corrupt index (requires rebuild)

### 2. `arqon heal`
**Purpose**: Attempt autonomous repair of the current failing state.
- **Flags**:
    - `--target <PATH>`: Limit healing to specific file/module.
    - `--dry-run`: Generate fix but do not apply.
    - `--rounds <N>`: Max attempts (capped at 2 by default).
- **Exit Codes**:
    - 0: Successfully repaired
    - 1: Repair failed (compile/test error remains)
    - 2: Safety gate violation (lint error in generated code)

### 3. `arqon ship`
**Purpose**: Run governed release pipeline.
- **Flags**:
    - `--dry-run`: Run checks but do not push/tag.
    - `--allow-dirty`: Skip uncommitted changes check (DEV ONLY).
- **Exit Codes**:
    - 0: Release successful
    - 1: Constitution violation (missing docs, debt, tests)

### 4. `arqon chat`
**Purpose**: Interactive TUI for querying the oracle.
- **Flags**:
    - `--cli`: Run in non-interactive stdout mode.
    - `--query <STRING>`: Direct query string.
