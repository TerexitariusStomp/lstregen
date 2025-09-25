#!/usr/bin/env bash
set -euo pipefail

# Ensure Docker is available
if ! command -v docker >/dev/null; then
  echo "Docker is required to run the optimizer. Install Docker and try again."
  exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Use CosmWasm workspace optimizer to build all contracts with reproducible settings
echo "Running CosmWasm workspace optimizer..."
docker run --rm -v "$PWD":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.6

# Verify expected artifacts exist
ART_DIR="$PROJECT_ROOT/artifacts"
echo "Artifacts directory: $ART_DIR"
ls -la "$ART_DIR" || true

# Expected outputs (crate package names become snake_case)
REQUIRED=(
  "regen_liquid_staking.wasm"
  "regen_validators.wasm"
  "regen_rewards.wasm"
)

MISSING=0
for f in "${REQUIRED[@]}"; do
  if [ ! -f "$ART_DIR/$f" ]; then
    echo "Missing artifact: $f"
    MISSING=1
  fi
done

if [ "$MISSING" -ne 0 ]; then
  echo "One or more artifacts are missing. Check optimizer output logs."
  exit 2
fi

echo "All artifacts built successfully:"
for f in "${REQUIRED[@]}"; do
  echo " - $ART_DIR/$f"
done