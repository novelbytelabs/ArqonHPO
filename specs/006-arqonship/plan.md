# Implementation Plan: ArqonShip DevSecOps

**Branch**: `006-arqonship` | **Date**: 2025-12-19 | **Spec**: [006-arqonship/spec.md](spec.md)
**Input**: Feature specification from `/specs/006-arqonship/spec.md`

## Summary

Implement **ArqonShip**, a unified Rust-based DevSecOps system providing a local "Codebase Oracle" (indexing), autonomous "Self-Healing CI" (LLM repair), and a "Governed Release" pipeline. It operates as a single binary (`arqon`) with no runtime dependencies, leveraging local AI models for privacy and speed.

## Technical Context

**Language/Version**: Rust 1.82+ (Edition 2021)
**Primary Dependencies**: 
- `tree-sitter` (parsing)
- `lance` / `lancedb` (vector storage, NEEDS CLARIFICATION on embedded usage)
- `candle` or `ort` (local AI inference, NEEDS CLARIFICATION on best choice)
- `ratatui` (TUI)
- `clap` (CLI)
- `reqwest` (GitHub API)
**Storage**: 
- Relational: SQLite (via `rusqlite` or `sqlx`?) for graph nodes/edges
- Vector: LanceDB (local file-based) for embeddings
**Testing**: `cargo test`, `insta` (snapshot testing for graph/CLI output)
**Target Platform**: Linux (x86_64), macOS (ARM64/x86_64), Windows (x86_64)
**Project Type**: Single binary CLI (`crates/ship`)
**Performance Goals**: <10s for full index (<50k LOC), <100MB RAM idle
**Constraints**: 
- 100% Offline capability (after model download)
- No Python runtime requirement for users
- Must run inside GitHub Actions Std Runner (2-core, 7GB RAM active)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **XVI. Codebase Oracle**:
  - [x] **Determinism**: Plan must ensure AST parsing and graph generation are deterministic (no map iteration order issues).
  - [x] **Privacy**: Default config MUST NOT use OpenAI/Anthropic. Local models only.
  - [x] **Schema**: Graph schema must be versioned.
- [x] **XVII. Self-Healing CI**:
  - [x] **Pinned Models**: Plan must verify support for exact model version pinning (e.g. `deepseek-coder-1.3b-v1`).
  - [x] **Safety Gates**: Healing loop must include syntax check + lint check steps.
  - [x] **Fail Closed**: Infinite loops forbidden (max 2 attempts).
- [x] **XVIII. CI/CD Automation**:
  - [x] **No Bypasses**: Automation flows must not use `[skip ci]`.
  - [x] **Rate Limits**: GitHub API calls must handle rate limits.
- [x] **XIX. CLI Contracts**:
  - [x] **Versioned Config**: `.arqon/config.toml` must include version field.

## Project Structure

### Documentation (this feature)

```text
specs/006-arqonship/
├── plan.md              # This file
├── research.md          # Output of Phase 0
├── data-model.md        # Output of Phase 1
├── quickstart.md        # Output of Phase 1
├── contracts/           # Output of Phase 1
└── tasks.md             # Output of Phase 2
```

### Source Code (repository root)

```text
crates/
├── ship/                # [NEW] The ArqonShip CLI binary
│   ├── src/
│   │   ├── main.rs
│   │   ├── oracle/      # Indexing & Graph logic
│   │   ├── heal/        # LLM interaction & Repair loops
│   │   ├── ship/        # Release pipeline logic
│   │   └── tui/         # Ratatui interface
│   └── Cargo.toml
└── core/                # Existing core crate (unchanged, referenced if needed)
```

**Structure Decision**: New crate `crates/ship` effectively acts as the "Tooling" layer, keeping the core HPO logic in `crates/core` pristine.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |
