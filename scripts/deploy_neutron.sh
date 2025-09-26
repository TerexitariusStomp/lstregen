#!/usr/bin/env bash
set -euo pipefail

NETWORK=${1:-test}
CHAIN_ID=${2:-test-1}
NODE_URL=${3:-tcp://neutron-node:26657}
DEPLOYER_KEY=${4:-funded}

echo "Deploying Regen Liquid Staking to $NETWORK ($CHAIN_ID) via $NODE_URL using key $DEPLOYER_KEY"

# Neutron CLI wrapper: use Docker image
NEUTRON_IMAGE="${NEUTRON_IMAGE:-neutron-node}"
KEYRING_DIR="${KEYRING_DIR:-$PWD/.neutron_keyring}"
mkdir -p "$KEYRING_DIR"

nd() {
   docker run --rm -e HOME=/home/neutron -u "$(id -u):$(id -g)" \
     --network neutron-testing \
     -v "$KEYRING_DIR":/home/neutron/.neutron \
     -v "$PWD":/work -w /work "$NEUTRON_IMAGE" \
     neutrond "$@"
 }
# Optional common args (e.g., --keyring-backend test)
RG_ARGS="--keyring-backend test --keyring-dir /home/neutron/.neutron"

# jq required locally
if ! command -v jq >/dev/null; then
  echo "Error: jq is required" >&2
  exit 1
fi

# cosmwasm-check wrapper: use local binary if present, otherwise run within optimizer image
cwcheck() {
  local f="$1"
  if command -v cosmwasm-check >/dev/null; then
    cosmwasm-check "$f" || { echo "cosmwasm-check failed on $f (continuing)"; return 0; }
  else
    echo "cosmwasm-check not available; skipping verification for $f"
    return 0
  fi
}

ART_DIR="artifacts"
mkdir -p "$ART_DIR"

if [ -f "$ART_DIR/regen_liquid_staking.wasm" ] && [ -f "$ART_DIR/regen_validators.wasm" ] && [ -f "$ART_DIR/regen_rewards.wasm" ]; then
  echo "Using existing artifacts in $ART_DIR"
else
  echo "Building optimized CosmWasm artifacts (workspace)..."
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/workspace-optimizer:0.16.0
fi

echo "Verifying contract artifacts..."
cwcheck artifacts/regen_liquid_staking.wasm
cwcheck artifacts/regen_validators.wasm
cwcheck artifacts/regen_rewards.wasm

if [ ! -f "$ART_DIR/cw20_base.wasm" ]; then
  echo "Fetching cw20-base optimized wasm..."
  if command -v curl >/dev/null; then
    curl -sSL -o "$ART_DIR/cw20_base.wasm" https://github.com/CosmWasm/cw-plus/releases/download/v1.1.2/cw20_base.wasm || true
  else
    docker run --rm -v "$PWD/$ART_DIR":/out curlimages/curl:8.8.0 -sSL -o /out/cw20_base.wasm https://github.com/CosmWasm/cw-plus/releases/download/v1.1.2/cw20_base.wasm || true
  fi
fi
if [ -f "$ART_DIR/cw20_base.wasm" ]; then
  cwcheck "$ART_DIR/cw20_base.wasm" || true
fi

echo "Storing Regen contracts..."
MAIN_CONTRACT_CODE_ID=$(nd $RG_ARGS tx wasm store artifacts/regen_liquid_staking.wasm \
  --from "$DEPLOYER_KEY" --chain-id "$CHAIN_ID" --node "$NODE_URL" \
  --fees 250000untrn,250000ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2 --gas 100000000 --output json | \
  jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

VALIDATOR_CONTRACT_CODE_ID=$(nd $RG_ARGS tx wasm store artifacts/regen_validators.wasm \
  --from "$DEPLOYER_KEY" --chain-id "$CHAIN_ID" --node "$NODE_URL" \
  --fees 250000untrn,250000ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2 --gas 100000000 --output json | \
  jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

REWARDS_CONTRACT_CODE_ID=$(nd $RG_ARGS tx wasm store artifacts/regen_rewards.wasm \
  --from "$DEPLOYER_KEY" --chain-id "$CHAIN_ID" --node "$NODE_URL" \
  --fees 250000untrn,250000ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2 --gas 100000000 --output json | \
  jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "Code IDs => main:$MAIN_CONTRACT_CODE_ID validators:$VALIDATOR_CONTRACT_CODE_ID rewards:$REWARDS_CONTRACT_CODE_ID"

echo "Preparing instantiate message for main contract..."
cat > instantiate_msg.json <<'EOF'
{
  "admin": "",
  "fee_rate": "0.05",
  "unbonding_period": 1814400,
  "max_validators": 20,
  "min_delegation": "1000000",
  "validators": [
    { "address": "neutronvaloper1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2", "weight": "1.0" }
  ]
}
EOF

if ! nd $RG_ARGS keys show "$DEPLOYER_KEY" -a >/dev/null 2>&1; then
  echo "Creating key $DEPLOYER_KEY"
  nd $RG_ARGS keys add "$DEPLOYER_KEY" --yes
fi
ADMIN_ADDR=$(nd $RG_ARGS keys show "$DEPLOYER_KEY" -a)
jq --arg admin "$ADMIN_ADDR" '.admin=$admin' instantiate_msg.json > instantiate_msg.tmp && mv instantiate_msg.tmp instantiate_msg.json

echo "Instantiating main contract..."
CONTRACT_ADDRESS=$(nd $RG_ARGS tx wasm instantiate "$MAIN_CONTRACT_CODE_ID" \
  "$(cat instantiate_msg.json)" \
  --from "$DEPLOYER_KEY" --chain-id "$CHAIN_ID" --node "$NODE_URL" \
  --fees 250000untrn,250000ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2 --gas 100000000 \
  --label "Neutron Liquid Staking v1.0" \
  --admin "$ADMIN_ADDR" \
  --output json | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')

echo "Main contract deployed at: $CONTRACT_ADDRESS"

CW20_CODE_ID=""
CW20_ADDRESS=""
if [ -f artifacts/cw20_base.wasm ]; then
  echo "Storing cw20-base code..."
  CW20_CODE_ID=$(nd $RG_ARGS tx wasm store artifacts/cw20_base.wasm \
    --from "$DEPLOYER_KEY" --chain-id "$CHAIN_ID" --node "$NODE_URL" \
    --fees 250000untrn,250000ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2 --gas 100000000 --output json | \
    jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
  echo "cw20-base code id: $CW20_CODE_ID"

  echo "Instantiating dNTRN cw20 with minter set to main contract..."
  cat > cw20_instantiate.json <<CW
{
  "name": "dNTRN",
  "symbol": "dNTRN",
  "decimals": 6,
  "initial_balances": [],
  "mint": { "minter": "$CONTRACT_ADDRESS" },
  "marketing": null
}
CW

  CW20_ADDRESS=$(nd $RG_ARGS tx wasm instantiate "$CW20_CODE_ID" \
    "$(cat cw20_instantiate.json)" \
    --from "$DEPLOYER_KEY" --chain-id "$CHAIN_ID" --node "$NODE_URL" \
    --fees 250000untrn,250000ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2 --gas 100000000 \
    --label "dNTRN CW20" \
    --admin "$ADMIN_ADDR" \
    --output json | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
  echo "dNTRN cw20 deployed at: $CW20_ADDRESS"

  echo "Updating main contract config with dregen_token..."
  nd $RG_ARGS tx wasm execute "$CONTRACT_ADDRESS" \
    "{\"update_config\":{\"dregen_token\":\"$CW20_ADDRESS\"}}" \
    --from "$DEPLOYER_KEY" --chain-id "$CHAIN_ID" --node "$NODE_URL" \
    --fees 250000untrn,250000ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2 --gas 100000000 -y
fi

cat > deployment_info.json <<EOF
{
  "network": "$NETWORK",
  "chain_id": "$CHAIN_ID",
  "contracts": {
    "main": { "code_id": $MAIN_CONTRACT_CODE_ID, "address": "$CONTRACT_ADDRESS" },
    "validators": { "code_id": $VALIDATOR_CONTRACT_CODE_ID },
    "rewards": { "code_id": $REWARDS_CONTRACT_CODE_ID },
    "cw20_base": { "code_id": ${CW20_CODE_ID:-null}, "address": ${CW20_ADDRESS:+\"$CW20_ADDRESS\"} }
  },
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "$ADMIN_ADDR"
}
EOF

echo "Deployment completed."
echo "Main contract address: $CONTRACT_ADDRESS"
echo "Deployment info saved to deployment_info.json"