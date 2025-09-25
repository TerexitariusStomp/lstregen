# Regen Liquid Staking — Deployment

This guide covers end-to-end environment preparation, building, optimizing, deploying, and validating the Regen Liquid Staking stack.

Related files:
- Workspace manifest: [`Cargo.toml`](../Cargo.toml)
- Main contract: [`contracts/regen-liquid-staking`](../contracts/regen-liquid-staking)
- Validator contract: [`contracts/regen-validators`](../contracts/regen-validators)
- Rewards contract: [`contracts/regen-rewards`](../contracts/regen-rewards)
- Scripts: [`scripts/optimize.sh`](../scripts/optimize.sh), [`scripts/deploy.sh`](../scripts/deploy.sh), [`scripts/test.sh`](../scripts/test.sh)
- Frontend: [`frontend`](../frontend)
- Indexer: [`indexer`](../indexer)
- Monitoring: [`monitoring/prometheus/config.yml`](../monitoring/prometheus/config.yml), [`monitoring/alerts/rules.yml`](../monitoring/alerts/rules.yml)

## 1) Prerequisites

- Rust and wasm target:
  - rustup default 1.75.0 (pinned via [`rust-toolchain`](../rust-toolchain))
  - rustup target add wasm32-unknown-unknown
- CosmWasm toolchain:
  - cargo install cargo-generate --features vendored-openssl
  - cargo install cargo-run-script
  - cargo install cosmwasm-check
- Docker (for optimizer images)
  - cosmwasm/rust-optimizer:0.12.6
  - cosmwasm/workspace-optimizer:0.12.6
- Regen CLI (or Ignite CLI)
- Node.js v18+ for frontend and indexer
- jq for JSON parsing in shell scripts

Note on Windows:
- The provided scripts are bash. Use WSL2 (Ubuntu) or Git Bash to run [`scripts/optimize.sh`](../scripts/optimize.sh) and [`scripts/deploy.sh`](../scripts/deploy.sh).

## 2) Build and Test

- Rust unit tests:
  - cargo test --workspace
- Dev build (debug):
  - cargo build --workspace --target wasm32-unknown-unknown
- Full test pipeline:
  - bash [`scripts/test.sh`](../scripts/test.sh)

## 3) Optimized Artifacts

Use the CosmWasm workspace optimizer for reproducible builds:

- bash [`scripts/optimize.sh`](../scripts/optimize.sh)

Expected outputs in ./artifacts:
- regen_liquid_staking.wasm
- regen_validators.wasm
- regen_rewards.wasm

Validate WASM artifacts:
- cosmwasm-check artifacts/regen_liquid_staking.wasm
- cosmwasm-check artifacts/regen_validators.wasm
- cosmwasm-check artifacts/regen_rewards.wasm

## 4) Chain Access and Keys

Ensure Regen CLI has a configured key:
- regen keys add deployer (or import via mnemonic)
- regen status --node https://rpc.redwood.regen.network:443
- regen q bank balances $(regen keys show deployer -a)

Fund the deployer account with uregen on your target network.

## 5) Deployment

Deploy using the provided script (stores code and instantiates main contract):

- bash [`scripts/deploy.sh`](../scripts/deploy.sh) testnet regen-redwood-1 https://rpc.redwood.regen.network:443 deployer

The script:
- Builds/optimizes with docker
- Runs cosmwasm-check
- Stores the main, validators, and rewards contract code
- Instantiates the main contract with a validator set
- Outputs deployment_info.json with code IDs and addresses

Adjust the validators list in the script or pass your own instantiate JSON if desired.

## 6) Post-Deployment Validation

Useful queries (replace CONTRACT and ADDRESS):

- Config:
  - regen q wasm contract-state smart CONTRACT '{"config":{}}' --node RPC
- State:
  - regen q wasm contract-state smart CONTRACT '{"state":{}}' --node RPC
- Exchange rate:
  - regen q wasm contract-state smart CONTRACT '{"exchange_rate":{}}' --node RPC
- Validators:
  - regen q wasm contract-state smart CONTRACT '{"validators":{}}' --node RPC

Sample execute messages:

- Stake (attached funds in uregen):
  - regen tx wasm execute CONTRACT '{"stake":{}}' --amount 1000000uregen --from deployer --chain-id CHAIN --node RPC -y
- Unbond:
  - regen tx wasm execute CONTRACT '{"unbond":{"dregen_amount":"500000"}}' --from deployer --chain-id CHAIN --node RPC -y
- Claim rewards (admin):
  - regen tx wasm execute CONTRACT '{"claim_rewards":{}}' --from deployer --chain-id CHAIN --node RPC -y

## 7) Frontend

Install and run:

- cd ./frontend
- npm install
- npm run start

Provide contract and RPC configuration to the app (env or component props). The main UI component is [`StakingInterface.tsx`](../frontend/src/components/StakingInterface.tsx).

## 8) Indexer

Install dependencies and run in dev:

- cd ./indexer
- npm install
- cp .env.example .env (create if not present) and set:
  - RPC_ENDPOINT=https://rpc.regen.network
  - CONTRACT_ADDRESS=regen1contract...
  - START_HEIGHT=0
  - DATABASE_URL=postgres://user:pass@localhost:5432/indexer
- npm run dev

Entities live in [`indexer/src/entities.ts`](../indexer/src/entities.ts) and logic in [`indexer/src/index.ts`](../indexer/src/index.ts).

## 9) Monitoring

- Prometheus:
  - config: [`monitoring/prometheus/config.yml`](../monitoring/prometheus/config.yml)
  - alerts: [`monitoring/alerts/rules.yml`](../monitoring/alerts/rules.yml)
- Adjust scrape targets and deploy Prometheus/Alertmanager in your infra.

## 10) Tips and Operations

- Fee rate guardrails: max 20% in instantiate/update (see [`contracts/regen-liquid-staking/src/contract.rs`](../contracts/regen-liquid-staking/src/contract.rs)).
- Pause/Resume for emergencies (admin-only).
- Rebalance is a placeholder for operator-driven delegation adjustments (see [`contracts/regen-liquid-staking/src/execute.rs`](../contracts/regen-liquid-staking/src/execute.rs)).
- CW20 dREGEN integration is expected to be set in config.dregen_token; you may instantiate CW20 separately and update via UpdateConfig.

## 11) Troubleshooting

- cosmwasm-check failures: ensure correct tool versions and rebuild via optimizer.
- RPC timeouts: try alternate RPC endpoints or increase gas/timeout flags.
- Unauthorized errors: confirm admin address and sender.
- Missing artifacts: verify docker bind mounts and rerun [`scripts/optimize.sh`](../scripts/optimize.sh).