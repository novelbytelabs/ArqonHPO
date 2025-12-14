# ArqonHPO Developer Commands
# ENFORCES: Canonical Environment (helios-gpu-118)

set shell := ["bash", "-c"]

# Paths mandated by Constitution
PYTHON := "/home/irbsurfer/miniconda3/envs/helios-gpu-118/bin/python"
CARGO := "/home/irbsurfer/miniconda3/envs/helios-gpu-118/bin/cargo"

# --- Default ---
default: test

# --- Build & Check ---
check:
    {{CARGO}} check --workspace

build:
    {{CARGO}} build --workspace

fmt:
    {{CARGO}} fmt

clippy:
    {{CARGO}} clippy --workspace -- -D warnings

# --- Testing ---
test-core:
    {{CARGO}} test -p arqonhpo-core

test-all:
    {{CARGO}} test --workspace

test: test-all

# --- Python Bindings ---
# Build and install into the active canonical env in editable mode
develop:
    {{PYTHON}} -m pip install --no-deps -e bindings/python

# Build wheels (release)
wheel:
    {{PYTHON}} -m maturin build --release -m bindings/python/Cargo.toml
