# Feature Logic Specification: ArqonShip DevSecOps

**Feature**: Implement ArqonShip: SOTA DevSecOps automation system with Codebase Oracle, Self-Healing CI, and Automated Release
**Feature Branch**: 006-arqonship
**Status**: Draft
**Last Updated**: 2025-12-19

## 1. Overview
ArqonShip is a State-of-the-Art (SOTA), fully automated, and reusable DevSecOps/CI/CD system designed to serve as the "autopilot" for Arqon development. It introduces a local-first "Codebase Oracle" for semantic understanding, an autonomous "Self-Healing CI" loop for automated failure repair, and a "Governed Release" pipeline that strictly enforces the Constitution's "DONE" standard. The system is built as a single Rust binary to minimize CI dependencies and ensure portability.

### 1.1 Context & Goal
Modern software development suffers from high cognitive load (understanding large codebases), repetitive maintenance (fixing minor CI breaks), and "release anxiety" (fear of breaking production). ArqonShip aims to eliminate these by providing:
1.  **Semantic Search:** Instant answers to "where is X defined?" and "what touches this global state?" via a local graph.
2.  **Autonomous Repair:** A system that doesn't just fail CI, but actively attempts to fix it using generative AI.
3.  **Governance as Code:** A release gate that mathematically proves a change meets quality standards before merge.

**Primary Goal:** Reduce developer toil by 50% through intelligent automation of search, repair, and release tasks.

### 1.2 In Scope
*   **Codebase Oracle (`arqon scan`)**:
    *   Local AST parsing (Tree-sitter) of Rust, Python, and TOML files.
    *   Dependency graph generation (Calls, Imports, Contains edges).
    *   Local embedding generation (MiniLM) and vector storage (LanceDB).
    *   CLI interface for querying the graph and vectors.
*   **Self-Healing CI (`arqon heal`)**:
    *   Parsing structured errors from CI logs (Rustc, Pytest).
    *   Construction of healing prompts with context.
    *   Interaction with local LLM (DeepSeek-Coder-1.3B) via `candle` or `ort`.
    *   "Whole Block Replacement" code application strategy.
    *   Verification loop (lint -> test -> commit).
*   **Automated Release (`arqon ship`)**:
    *   Pre-flight checklist enforcement (Constitution 8-Pillar check).
    *   Automated version bumping (SemVer).
    *   Changelog generation.
    *   GitHub PR automation.
*   **CLI Interface**: Unified `arqon` binary with subcommands.

### 1.3 Out of Scope
*   **Cloud LLM Integration during Indexing**: Strictly local-first for privacy (per Constitution XVI.3).
*   **Distributed CI Runner**: ArqonShip runs *inside* existing CI runners (GitHub Actions), it does not replace the runner infrastructure.
*   **Non-Code Repair**: Fixing logical business errors or architectural flaws is out of scope for the AI; focus is on syntax, types, and clear runtime errors.

## 2. User Scenarios
*   **Developer Onboarding**: "As a new dev, I run `arqon chat 'How does the retry logic work?'` and get a referenced answer citing `crates/core/src/retry.rs`, without waiting for a senior engineer."
*   **CI Self-Correction**: "As a maintainer, I push a typo that breaks the build. Instead of a failure notification, I get a notification that ArqonShip fixed the typo, committed the fix, and the build passed."
*   **Confident Release**: "As a release manager, I run `arqon ship`. The system verifies 100% test coverage, updated docs, and no debt without TTL, then auto-generated the release PR. I merge with confidence."

## 3. Functional Requirements

### 3.1 Codebase Oracle (`arqon scan`)
*   **REQ-3.1.1 (Graph Generation)**: The system MUST generate a directed graph of the codebase where nodes are Files, Modules, Structs, Functions, and Enums, and edges represent Imports, Calls, and Contains relationships.
*   **REQ-3.1.2 (Determinism)**: The graph generation MUST be deterministic. Identical source code MUST produce an identical graph structure and hash, regardless of the machine or time of day (Constitution XVI.1).
*   **REQ-3.1.3 (Vector Indexing)**: The system MUST generate vector embeddings for all Function and Struct nodes using a local embedding model (e.g., `all-MiniLM-L6-v2`) and store them in a local LanceDB index.
*   **REQ-3.1.4 (Query Interface)**: The CLI MUST provide a `query` command that accepts natural language and returns relevant code blocks based on vector similarity and graph connectivity.

### 3.2 Self-Healing CI (`arqon heal`)
*   **REQ-3.2.1 (Error Parsing)**: The system MUST parse standard error formats (Rustc JSON, Pytest XML) to extract the failing file, line number, and error message.
*   **REQ-3.2.2 (Context Retrieval)**: The system MUST retrieve the relevant source code block and its immediate dependencies (signatures of called functions) from the Oracle to form the prompt context.
*   **REQ-3.2.3 (AI Generation)**: The system MUST prompt a local LLM (DeepSeek-Coder-1.3B) to rewrite the failing code block. The prompt MUST be version-controlled (Constitution XVII.1).
*   **REQ-3.2.4 (Safe Application)**: The system MUST apply the fix using "Whole Block Replacement" (replacing the entire function body) rather than applying a diff.
*   **REQ-3.2.5 (Verification Gate)**: The generated fix MUST pass syntax checks and `cargo clippy` / `ruff` before being attempted in tests (Constitution XVII.2).

### 3.3 Automated Release (`arqon ship`)
*   **REQ-3.3.1 (Constitution Check)**: The release process MUST fail if any of the following are true:
    *   Uncommitted changes exist.
    *   Tests are failing locally.
    *   `TD-###` debt tags exist without a valid future TTL.
    *   Public API changes are missing documentation updates.
*   **REQ-3.3.2 (Version Bump)**: The system MUST automatically determine the next SemVer version based on Conventional Commits in the history.
*   **REQ-3.3.3 (Changelog)**: The system MUST generate a changelog entry grouping changes by type (Feature, Fix, Perf) and filtering out "chore" commits.

## 4. Non-Functional Requirements
*   **NFR-4.1 (Performance)**: `arqon scan` MUST complete a full index of the current ArqonHPO codebase (<50k lines) in under 10 seconds on a standard dev machine.
*   **NFR-4.2 (Privacy)**: No source code or embeddings shall be transmitted to any third-party API during `arqon scan` (Constitution XVI.3).
*   **NFR-4.3 (Memory Safety)**: The CLI MUST be essentially memory-leak free and panic-free (Rust guarantees).
*   **NFR-4.4 (CI Footprint)**: The ArqonShip binary and its models (quantized) MUST fit within the 16GB RAM limit of standard GitHub Actions runners.

## 5. Data Model
*   **Entities**:
    *   `GraphNode`: {id, type, path, start_line, end_line, signature_hash}
    *   `GraphEdge`: {source_id, target_id, type}
    *   `HealAttempt`: {run_id, file_path, error_msg, prompt_hash, diff_hash, outcome}
*   **Storage**:
    *   Code Graph: `~/.arqon/graph.json` (Phase 1)
    *   Vector Index: `~/.arqon/vectors.lance/`
    *   Config: `.arqon/config.toml`

## 6. Success Criteria
*   **SC-6.1**: `arqon scan` successfully indexes the entire ArqonHPO repo and answers a query like "Find the SPSA optimizer" correctly in top-3 results.
*   **SC-6.2**: `arqon heal` successfully repairs a synthetic syntax error (missing semicolon) and a simple type error fully autonomously in a CI environment.
*   **SC-6.3**: `arqon ship` successfully prevents a release when a dummy "TODO" without a debt tag is introduced.
*   **SC-6.4**: The system operates entirely offline (after initial model download) with zero API calls to external AI providers.

## 7. Assumptions & Risks
*   **Assumption**: 1.3B parameter models are sufficient for syntax/type fixing but may struggle with complex logical bugs.
*   **Assumption**: GitHub Actions runners have sufficient CPU/AVX2 support to run the quantized models at acceptable speed (inference < 30s).
*   **Risk**: LLM hallucination could introduce subtle bugs. **Mitigation**: Strict verification gates (REQ-3.2.5) and human review of all AI commits.

## 8. Requirements Traceability Matrix
(To be filled during Planning phase)
