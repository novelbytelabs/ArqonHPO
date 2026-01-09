#!/bin/bash
set -e

echo "==> Running Cargo Format..."
cargo fmt --all

echo "==> Running Cargo Clippy Fix..."
# Allow dirty/staged to run fix even if there are uncommitted changes
cargo clippy --workspace --fix --allow-dirty --allow-staged

echo "✅ Code auto-fix complete!"
