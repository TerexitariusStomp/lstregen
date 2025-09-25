#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "[1/3] Running Rust unit tests..."
cargo test --workspace

echo "[2/3] Building contracts (debug)..."
cargo build --workspace --target wasm32-unknown-unknown

echo "[3/3] Optimizing contracts (release, reproducible)..."
bash ./scripts/optimize.sh

echo "All tests and builds completed successfully."