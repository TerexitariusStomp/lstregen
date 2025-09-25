# Regen Network Liquid Staking — Monorepo

End-to-end implementation scaffold of a CosmWasm-based liquid staking protocol for Regen Network, including smart contracts, types, scripts, frontend, indexer, monitoring, and docs.

- Contracts
  - Main liquid staking: [contracts/regen-liquid-staking](contracts/regen-liquid-staking)
  - Validator management: [contracts/regen-validators](contracts/regen-validators)
  - Rewards accounting: [contracts/regen-rewards](contracts/regen-rewards)
- Shared
  - Types: [packages/regen-types](packages/regen-types)
  - Testing utilities: [packages/regen-testing](packages/regen-testing)
- Tooling
  - Scripts: [scripts](scripts)
  - Frontend: [frontend](frontend)
  - Indexer: [indexer](indexer)
  - Monitoring: [monitoring](monitoring)
- Docs
  - Architecture: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
  - API Reference: [docs/API.md](docs/API.md)
  - Deployment Guide: [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)

## Prerequisites

- Rust (pinned via [rust-toolchain](rust-toolchain))
- wasm32 target: `rustup target add wasm32-unknown-unknown`
- CosmWasm tooling: `cargo-generate`, `cosmwasm-check`, `cargo-run-script`
- Docker (for optimizer images)
- Node.js 18+ (frontend/indexer)
- jq (for scripts)
- Regen CLI and funded key for deployments

## Quick Start

1) Install prerequisites (Rust toolchain, wasm target, Docker).
2) Build and test workspace:
   - `cargo test --workspace`
   - `cargo build --workspace --target wasm32-unknown-unknown`
3) Optimize contracts for deployment:
   - `bash scripts/optimize.sh`
4) Deploy to a Regen network (adjust args as needed):
   - `bash scripts/deploy.sh testnet regen-redwood-1 https://rpc.redwood.regen.network:443 deployer`

Deployment outputs are validated with `cosmwasm-check` and summarized in `deployment_info.json`.

## Contracts

Main contract modules:
- State: [state.rs](contracts/regen-liquid-staking/src/state.rs)
- Messages: [msg.rs](contracts/regen-liquid-staking/src/msg.rs)
- Entrypoints: [contract.rs](contracts/regen-liquid-staking/src/contract.rs)
- Execute: [execute.rs](contracts/regen-liquid-staking/src/execute.rs)
- Query: [query.rs](contracts/regen-liquid-staking/src/query.rs)
- Math: [math.rs](contracts/regen-liquid-staking/src/math.rs)
- Errors: [error.rs](contracts/regen-liquid-staking/src/error.rs)
- Helpers: [helpers.rs](contracts/regen-liquid-staking/src/helpers.rs)
- Unit tests: [tests.rs](contracts/regen-liquid-staking/src/tests.rs)

Validator management:
- [contracts/regen-validators/src/contract.rs](contracts/regen-validators/src/contract.rs)
- [contracts/regen-validators/src/msg.rs](contracts/regen-validators/src/msg.rs)
- [contracts/regen-validators/src/state.rs](contracts/regen-validators/src/state.rs)

Rewards accounting:
- [contracts/regen-rewards/src/contract.rs](contracts/regen-rewards/src/contract.rs)
- [contracts/regen-rewards/src/msg.rs](contracts/regen-rewards/src/msg.rs)
- [contracts/regen-rewards/src/state.rs](contracts/regen-rewards/src/state.rs)

## Scripts

- Optimize: [scripts/optimize.sh](scripts/optimize.sh)
- Deploy: [scripts/deploy.sh](scripts/deploy.sh)
- Test pipeline: [scripts/test.sh](scripts/test.sh)

Run test pipeline:
```
bash scripts/test.sh
```

## Frontend

React-based staking interface:
- Component: [frontend/src/components/StakingInterface.tsx](frontend/src/components/StakingInterface.tsx)
- Install/start:
  - `cd frontend`
  - `npm install`
  - `npm run start`

Pass `contractAddress`, `rpcEndpoint`, and optional `chainId` to the component.

## Indexer

Typescript indexer for capturing contract wasm events:
- Entry: [indexer/src/index.ts](indexer/src/index.ts)
- Entities: [indexer/src/entities.ts](indexer/src/entities.ts)
- Handlers: [indexer/src/handlers/index.ts](indexer/src/handlers/index.ts)

Setup:
```
cd indexer
npm install
npm run dev
```

Configure via environment variables:
- RPC_ENDPOINT, CONTRACT_ADDRESS, START_HEIGHT, DATABASE_URL

## Monitoring

- Prometheus config: [monitoring/prometheus/config.yml](monitoring/prometheus/config.yml)
- Alerts: [monitoring/alerts/rules.yml](monitoring/alerts/rules.yml)

Adjust scrape targets and deploy Prometheus/Alertmanager in your environment.

## Development Notes

- dREGEN CW20 address is stored in config and used for mint/burn; instantiate CW20 externally or extend the flow to instantiate and store it at genesis.
- Exchange rate/rewards query to staking module is placeholder; integrate with chain distribution queries in production.
- Rebalance is a placeholder for operator-controlled delegation logic.

## License

Apache-2.0