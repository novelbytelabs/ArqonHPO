# Research & Technical Decisions: ArqonShip

**Phase 0 Output** | **Date**: 2025-12-19
**Feature**: ArqonShip DevSecOps System

## 1. Local AI Inference Runtime
**Decision**: Use **Candle** (HuggingFace).
- **Rationale**: 
    - **Pure Rust**: No heavy C++ shared libraries (unlike `ort`), enabling a simpler build process and smaller binary.
    - **Integration**: Native support for `safetensors` and `tokenizers` crates.
    - **Performance**: Sufficient for `all-MiniLM` (embeddings) and `deepseek-coder-1.3b` (inference) on CPU/Metal.
    - **Deployment**: Zero-dependency static linking possible.
- **Alternatives Considered**: 
    - `ort` (ONNX Runtime): Faster, but distribution is complex (DLL hell) and adds C++ weight. Use as fallback if Candle fails on specific HW.
    - `tch-rs` (LibTorch): Too heavy (GBs of deps), requires Python/C++ lib installation.

## 2. Vector Storage Engine
**Decision**: Use **LanceDB** (Embedded Rust SDK).
- **Rationale**:
    - **Embedded**: Runs in-process (like SQLite), no separate server required.
    - **Storage**: Uses efficient Lance file format, easy to manage on local disk (`~/.arqon/vectors.lance`).
    - **Query**: Fast approximate nearest neighbor (ANN) search.
- **Alternatives Considered**: 
    - `duckdb` (with `vss`): Good, but vector extension distribution for all platforms is tricky.
    - `hnswlib-rs`: In-memory only, persistence is manual. LanceDB handles disk persistence better.

## 3. Codebase Graph Storage
**Decision**: Use **SQLite** (via `rusqlite`).
- **Rationale**:
    - **Determinism**: Single file, transactional, consistent.
    - **Query**: Complex relational queries (Find all functions called by X that are defined in Y) are trivial in SQL.
    - **Portability**: Ubiquitous support.
- **Alternatives Considered**:
    - `petgraph` serialization: Fast but requires loading entire graph into RAM. SQLite allows partial loading (better for large repos).

## 4. Models
**Decision**:
- **Embeddings**: `sentence-transformers/all-MiniLM-L6-v2` (Quantized to q4_k or similar if possible, else fp16). Small (~80MB), fast.
- **Healing LLM**: `deepseek-ai/deepseek-coder-1.3b-instruct` (Quantized GGUF/Generic via Candle). Fits in <2GB RAM.
- **Rationale**:
    - These models represent the best "bang for buck" for running on standard CI runners (2vCPU, 7GB RAM).

## 5. Technical Stack Summary
- **Language**: Rust 1.82+
- **CLI**: `clap`
- **TUI**: `ratatui`
- **AI**: `candle-core`, `candle-nn`, `candle-transformers`
- **Vectors**: `lancedb`
- **Graph**: `rusqlite`
- **Parsing**: `tree-sitter`
