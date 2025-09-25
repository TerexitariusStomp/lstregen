# Regen Liquid Staking — Architecture

This document describes the high-level architecture of the Regen Liquid Staking protocol and how the repository is organized.

## Workspace Layout

- Contracts (CosmWasm):
  - Main liquid staking: [`contracts/regen-liquid-staking`](../contracts/regen-liquid-staking)
    - Core modules:
      - State: [`state.rs`](../contracts/regen-liquid-staking/src/state.rs)
      - Messages: [`msg.rs`](../contracts/regen-liquid-staking/src/msg.rs)
      - Entrypoints: [`contract.rs`](../contracts/regen-liquid-staking/src/contract.rs)
      - Execute handlers: [`execute.rs`](../contracts/regen-liquid-staking/src/execute.rs)
      - Query handlers: [`query.rs`](../contracts/regen-liquid-staking/src/query.rs)
      - Math utils: [`math.rs`](../contracts/regen-liquid-staking/src/math.rs)
      - Errors: [`error.rs`](../contracts/regen-liquid-staking/src/error.rs)
      - Helpers: [`helpers.rs`](../contracts/regen-liquid-staking/src/helpers.rs)
      - Crate exports: [`lib.rs`](../contracts/regen-liquid-staking/src/lib.rs)
  - Validator management: [`contracts/regen-validators`](../contracts/regen-validators)
    - State: [`state.rs`](../contracts/regen-validators/src/state.rs)
    - Messages: [`msg.rs`](../contracts/regen-validators/src/msg.rs)
    - Entrypoints: [`contract.rs`](../contracts/regen-validators/src/contract.rs)
  - Rewards management: [`contracts/regen-rewards`](../contracts/regen-rewards)
    - State: [`state.rs`](../contracts/regen-rewards/src/state.rs)
    - Messages: [`msg.rs`](../contracts/regen-rewards/src/msg.rs)
    - Entrypoints: [`contract.rs`](../contracts/regen-rewards/src/contract.rs)
- Shared Types:
  - [`packages/regen-types`](../packages/regen-types)
    - Entry: [`lib.rs`](../packages/regen-types/src/lib.rs)
    - Validator types: [`validator.rs`](../packages/regen-types/src/validator.rs)
    - Staking types: [`staking.rs`](../packages/regen-types/src/staking.rs)
    - Rewards types: [`rewards.rs`](../packages/regen-types/src/rewards.rs)
- Testing Utilities:
  - [`packages/regen-testing`](../packages/regen-testing)
    - Entry: [`lib.rs`](../packages/regen-testing/src/lib.rs)
    - Mock querier: [`mock_querier.rs`](../packages/regen-testing/src/mock_querier.rs)
- Frontend:
  - [`frontend`](../frontend)
  - Main component: [`StakingInterface.tsx`](../frontend/src/components/StakingInterface.tsx)
- Indexer:
  - [`indexer`](../indexer)
  - App entry: [`src/index.ts`](../indexer/src/index.ts)
  - Entities: [`src/entities.ts`](../indexer/src/entities.ts)
- Monitoring:
  - Prometheus: [`monitoring/prometheus/config.yml`](../monitoring/prometheus/config.yml)
  - Alerts: [`monitoring/alerts/rules.yml`](../monitoring/alerts/rules.yml)
- Scripts:
  - Deploy: [`scripts/deploy.sh`](../scripts/deploy.sh)
  - Optimize: [`scripts/optimize.sh`](../scripts/optimize.sh)
  - Test: [`scripts/test.sh`](../scripts/test.sh)
- Tests:
  - Unit: in-contract modules, e.g., [`tests.rs`](../contracts/regen-liquid-staking/src/tests.rs)
  - Integration: [`tests/integration/mod.rs`](../tests/integration/mod.rs)

## Components and Responsibilities

### Main Liquid Staking Contract

- Initialization and config management: [`contract.rs`](../contracts/regen-liquid-staking/src/contract.rs)
- Staking, unbonding, claim-unbonding, rewards, rebalance, pause/resume, update-config: [`execute.rs`](../contracts/regen-liquid-staking/src/execute.rs)
- Queries: config, state, exchange rate, validators, unbonding, simulation helpers: [`query.rs`](../contracts/regen-liquid-staking/src/query.rs)
- State model: [`state.rs`](../contracts/regen-liquid-staking/src/state.rs)
- Math (exchange rate, fee, distribution, APR): [`math.rs`](../contracts/regen-liquid-staking/src/math.rs)

Key flows:
- Stake:
  1. Validate min stake and paused status.
  2. Compute exchange rate and fee, mint dREGEN via CW20.
  3. Delegate REGEN to active validators by weight.
- Unbond:
  1. Burn dREGEN, compute REGEN redemption and fee.
  2. Undelegate proportionally, create unbonding request with completion time.
- Claim Rewards:
  - Admin-only; withdraw delegator rewards across validators (distribution messages).
- Rebalance:
  - Admin-only; placeholder to implement delegation rebalancing logic.

### Validator Management Contract

- Owns the set of validators and their attributes.
- Admin operations: add/remove, activate/deactivate, update weight/commission, transfer admin.

### Rewards Management Contract

- Records rewards and claims for audit/analytics.
- Admin/distributor roles to prevent spoofing.

### Frontend

- Displays overall staking stats and exchange rate in [`StakingInterface.tsx`](../frontend/src/components/StakingInterface.tsx).
- Keplr wallet integration, stake and unbond flows with simple UX.

### Indexer

- Polls chain for WASM events for the main contract.
- Persists Stake/Unbond/Reward events into Postgres using TypeORM entities.

## Security and Safety Considerations

- Admin-controlled parameters (fee caps, pause switch).
- Fee rate capped at 20% in instantiate/update.
- Validator set bounds with `max_validators`.
- Unbonding creates explicit requests and delayed claims.
- Exchange rate computed from total staked, supply, and rewards (query placeholder to staking module).
- Further hardening recommended:
  - Add allowlist for CW20 dREGEN contract or instantiate it and store address.
  - Implement slashing/uptime/commission monitoring feedback from `regen-validators`.

## Build and Release

- Unit/Integration tests:
  - `bash` [`scripts/test.sh`](../scripts/test.sh)
- Optimized builds:
  - `bash` [`scripts/optimize.sh`](../scripts/optimize.sh)
- Deployment:
  - `bash` [`scripts/deploy.sh`](../scripts/deploy.sh)
