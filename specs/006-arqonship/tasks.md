# Implementation Tasks: ArqonShip DevSecOps

**Branch**: `006-arqonship` | **Date**: 2025-12-19
**Spec**: [006-arqonship/spec.md](spec.md) | **Plan**: [006-arqonship/plan.md](plan.md)

## Summary
Total Tasks: 41
- Phase 1: Setup (5 tasks)
- Phase 2: [US1] Codebase Oracle (14 tasks)
- Phase 3: [US2] Self-Healing CI (12 tasks)
- Phase 4: [US3] Automated Release (6 tasks)
- Phase 5: Polish & TUI (4 tasks)

## Phase 1: Setup
Goal: Initialize the `ship` crate and core dependencies.

- [x] T001 Create `crates/ship` project structure and workspace membership in `Cargo.toml`
- [x] T002 Add dependencies to `crates/ship/Cargo.toml` (clap, ratatui, rusqlite, lance, candle-core, tree-sitter)
- [x] T003 Implement `Config` struct and versioned TOML loader in `crates/ship/src/config.rs`
- [x] T004 Implement `arqon init` command to generate default `.arqon/config.toml` in `crates/ship/src/main.rs`
- [x] T005 Create SQL schema migration for Graph and Audit DB in `crates/ship/src/oracle/schema.rs`

## Phase 2: [US1] Codebase Oracle
Goal: Implement `arqon scan` and Query Interface (Graph + Vectors).

### Graph Generation (SQLite)
- [x] T006 [US1] Implement Tree-sitter parser for Rust in `crates/ship/src/oracle/parser.rs`
- [x] T007 [US1] Implement Tree-sitter parser for Python in `crates/ship/src/oracle/parser_py.rs`
- [x] T008 [US1] Implement `GraphBuilder` to extract Nodes (Functions, Structs) in `crates/ship/src/oracle/graph.rs`
- [x] T009 [US1] Implement Edge extractor (Calls, Imports) in `crates/ship/src/oracle/edges.rs`
- [x] T010 [US1] Implement SQLite storage layer (`Node`, `Edge` dao) in `crates/ship/src/oracle/store.rs`
- [x] T011 [US1] Implement deterministic content hashing for nodes in `crates/ship/src/oracle/hash.rs`

### Vector Indexing (LanceDB)
- [x] T012 [P] [US1] Implement `EmbeddingModel` trait and Candle implementation (MiniLM) in `crates/ship/src/oracle/embed.rs`
- [x] T013 [P] [US1] Implement LanceDB storage layer for code vectors in `crates/ship/src/oracle/vector_store.rs`
- [x] T014 [US1] Implement `arqon scan` command showing progress bar in `crates/ship/src/oracle/mod.rs`
- [x] T015 [P] [US1] Implement incremental scan logic (hash check vs DB) in `crates/ship/src/oracle/incremental.rs`

### Query Interface
- [x] T016 [US1] Implement `QueryEngine` combining SQL (graph) and Vector search in `crates/ship/src/oracle/query.rs`
- [x] T017 [US1] Implement `arqon chat --cli` subcommand in `crates/ship/src/main.rs`
- [x] T018 [US1] Add test: Verify graph extraction on `crates/core/src/lib.rs` (snapshot test)
- [ ] T019 [US1] Add test: Verify vector search returns relevant snippet for "optimizer"

## Phase 3: [US2] Self-Healing CI
Goal: Implement `arqon heal` autonomous repair loop.

### Analysis & Context
- [ ] T020 [US2] Implement `LogParser` for `cargo test` JSON output in `crates/ship/src/heal/parser_rust.rs`
- [ ] T021 [US2] Implement `LogParser` for `pytest` XML/text output in `crates/ship/src/heal/parser_py.rs`
- [ ] T022 [US2] Implement `ContextBuilder` to fetch relevant graph nodes for failing file in `crates/ship/src/heal/context.rs`

### AI & Code Gen (Local LLM)
- [ ] T023 [P] [US2] Implement `LlmClient` trait and Candle implementation (DeepSeek-1.3B) in `crates/ship/src/heal/llm.rs`
- [ ] T024 [US2] Create prompt templates for Rust/Python repair in `crates/ship/src/heal/prompts.rs`
- [ ] T025 [US2] Implement `HealingLoop` state machine (Analyze -> Prompt -> Gen -> Apply) in `crates/ship/src/heal/loop.rs`

### Safety & Governance
- [ ] T026 [US2] Implement "Whole Block Replacement" logic to apply LLM fixes in `crates/ship/src/heal/apply.rs`
- [ ] T027 [US2] Implement `VerificationGate` (check compile, lint, test) in `crates/ship/src/heal/verify.rs`
- [ ] T028 [US2] Implement Audit Logging to `~/.arqon/audit.db` in `crates/ship/src/heal/audit.rs`
- [ ] T029 [US2] Implement `arqon heal` command in `crates/ship/src/main.rs`
- [ ] T030 [US2] Add test: Synthetic repair of missing semicolon (Rust)
- [ ] T031 [US2] Add test: Synthetic repair of type error (Rust)

## Phase 4: [US3] Automated Release
Goal: Implement `arqon ship` governed pipeline.

- [ ] T032 [US3] Implement `ConstitutionCheck` (Clean git, Passing tests, No debt tags) in `crates/ship/src/ship/checks.rs`
- [ ] T033 [US3] Implement Conventional Commits parser in `crates/ship/src/ship/commits.rs`
- [ ] T034 [US3] Implement SemVer calculator and Changelog generator in `crates/ship/src/ship/version.rs`
- [ ] T035 [US3] Implement GitHub PR creation via `reqwest` in `crates/ship/src/ship/github.rs`
- [ ] T036 [US3] Implement `arqon ship` command in `crates/ship/src/main.rs`
- [ ] T037 [US3] Add test: Verify `ship` fails on "TODO" existence without debt tag

## Phase 5: Polish & TUI
Goal: Interactive TUI and final integration.

- [ ] T038 [P] Implement Ratatui-based interactive chat UI in `crates/ship/src/tui/chat.rs`
- [ ] T039 Implement CI Workflow YAML `arqon-heal.yml` to run `arqon heal` on failure
- [ ] T040 Performance pass: Ensure `scan` < 10s on ArqonHPO
- [ ] T041 Write user documentation in `docs/arqon-ship.md`

## Dependencies
- Phase 2 (Oracle) is prerequisite for Phase 3 (Heal) context retrieval.
- Phase 3 (Heal) and Phase 4 (Ship) can run in parallel.
