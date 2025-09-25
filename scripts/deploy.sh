#!/bin/bash

set -e

# Configuration
NETWORK=${1:-testnet}
CHAIN_ID=${2:-regen-redwood-1}
NODE_URL=${3:-https://rpc.redwood.regen.network:443}
DEPLOYER_KEY=${4:-deployer}

echo "Deploying to $NETWORK ($CHAIN_ID)"

# Build and optimize contracts
echo "Building contracts..."
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.6

# Verify contract optimization
echo "Verifying contract artifacts..."
cosmwasm-check artifacts/regen_liquid_staking.wasm
cosmwasm-check artifacts/regen_validators.wasm
cosmwasm-check artifacts/regen_rewards.wasm

# Deploy contracts
echo "Storing contract code..."
MAIN_CONTRACT_CODE_ID=$(regen tx wasm store artifacts/regen_liquid_staking.wasm \
  --from $DEPLOYER_KEY \
  --chain-id $CHAIN_ID \
  --node $NODE_URL \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode block \
  --output json | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

VALIDATOR_CONTRACT_CODE_ID=$(regen tx wasm store artifacts/regen_validators.wasm \
  --from $DEPLOYER_KEY \
  --chain-id $CHAIN_ID \
  --node $NODE_URL \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode block \
  --output json | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "Main contract code ID: $MAIN_CONTRACT_CODE_ID"
echo "Validator contract code ID: $VALIDATOR_CONTRACT_CODE_ID"

# Create instantiate message
cat > instantiate_msg.json << EOF
{
  "admin": "$(regen keys show $DEPLOYER_KEY -a)",
  "fee_rate": "0.05",
  "unbonding_period": 1814400,
  "max_validators": 20,
  "min_delegation": "1000000",
  "validators": [
    {
      "address": "regenvaloper1n3mhyp9fvcmuu8l0q8qvjy07x0rql8q4ucmxys",
      "weight": "0.2"
    },
    {
      "address": "regenvaloper1ss2f0nl7sn42x8x0d337mj9welzml8h0f5erue",
      "weight": "0.2"
    },
    {
      "address": "regenvaloper1ceunjpth8nds7sfmfd9yjmh97vxmuzth4z4gkmmgvjdxfdz",
      "weight": "0.2"
    },
    {
      "address": "regenvaloper1vys9dreue4e8xrga2zmuzth4z4gkmmgvjdxfdz",
      "weight": "0.2"
    },
    {
      "address": "regenvaloper105g89nqllu33nend0ce5eup4zxn0d4kfr2v7w8",
      "weight": "0.2"
    }
  ]
}
EOF

# Instantiate main contract
echo "Instantiating main contract..."
CONTRACT_ADDRESS=$(regen tx wasm instantiate $MAIN_CONTRACT_CODE_ID \
  "$(cat instantiate_msg.json)" \
  --from $DEPLOYER_KEY \
  --chain-id $CHAIN_ID \
  --node $NODE_URL \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode block \
  --label "Regen Liquid Staking v1.0" \
  --admin "$(regen keys show $DEPLOYER_KEY -a)" \
  --output json | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')

echo "Contract deployed at: $CONTRACT_ADDRESS"

# Save deployment info
cat > deployment_info.json << EOF
{
  "network": "$NETWORK",
  "chain_id": "$CHAIN_ID",
  "contracts": {
    "main": {
      "code_id": $MAIN_CONTRACT_CODE_ID,
      "address": "$CONTRACT_ADDRESS"
    },
    "validators": {
      "code_id": $VALIDATOR_CONTRACT_CODE_ID
    }
  },
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "$(regen keys show $DEPLOYER_KEY -a)"
}
EOF

echo "Deployment completed successfully!"
echo "Contract address: $CONTRACT_ADDRESS"
echo "Deployment info saved to deployment_info.json"