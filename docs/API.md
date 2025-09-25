# Regen Liquid Staking — API

This document defines the on-chain contract interfaces (instantiate, execute, query), expected events/attributes, and example payloads for the main liquid staking contract. See source definitions at:
- [contracts/regen-liquid-staking/src/msg.rs](../contracts/regen-liquid-staking/src/msg.rs)
- [contracts/regen-liquid-staking/src/execute.rs](../contracts/regen-liquid-staking/src/execute.rs)
- [contracts/regen-liquid-staking/src/query.rs](../contracts/regen-liquid-staking/src/query.rs)
- [contracts/regen-liquid-staking/src/contract.rs](../contracts/regen-liquid-staking/src/contract.rs)

The validator and rewards helper contracts are also included:
- [contracts/regen-validators/src/msg.rs](../contracts/regen-validators/src/msg.rs)
- [contracts/regen-rewards/src/msg.rs](../contracts/regen-rewards/src/msg.rs)

## Types

Validator parameters:
```json
{
  "address": "regenvaloper1...",
  "weight": "0.20"
}
```

## Instantiate

Route: wasm/instantiate

Message:
```json
{
  "admin": "regen1...",
  "fee_rate": "0.05",
  "unbonding_period": 1814400,
  "max_validators": 20,
  "min_delegation": "1000000",
  "validators": [
    { "address": "regenvaloper1...", "weight": "0.2" }
  ]
}
```

Constraints:
- fee_rate <= 0.20
- validators length <= max_validators
- min_delegation in uregen

State initialized in:
- [contracts/regen-liquid-staking/src/state.rs](../contracts/regen-liquid-staking/src/state.rs)

## Execute

Route: wasm/execute

Messages:

1) Stake
```json
{ "stake": {} }
```
- Funds: [{ denom: "uregen", amount: "<uamount>" }]
- Effects:
  - Delegates pro-rata to active validators by weight
  - Mints dREGEN via CW20 (CW20 contract address stored in config.dregen_token)
  - Sends fee to admin (treasury) if fee_rate > 0
- Emits attributes:
  - action=stake
  - staker=<addr>
  - regen_amount=<uamount>
  - dregen_amount=<uamount>
  - exchange_rate=<decimal>
  - fee_amount=<uamount>

2) Unbond
```json
{ "unbond": { "dregen_amount": "5000000" } }
```
- Burns dREGEN from sender via CW20 BurnFrom
- Schedules undelegation from validators pro-rata
- Creates unbonding request entry
- Emits:
  - action=unbond
  - user=<addr>
  - dregen_amount=<uamt>
  - regen_amount=<uamt>
  - unbonding_id=<id>
  - completion_time=<unix_seconds>
  - exchange_rate=<decimal>

3) ClaimUnbonding
```json
{ "claim_unbonding": { "unbonding_id": 1 } }
```
- Preconditions:
  - Sender owns the unbonding request
  - env.block.time >= completion_time
- Effects:
  - Sends REGEN to user (uregen)
  - Removes request; decrements pending_unbonding
- Emits:
  - action=claim_unbonding
  - user=<addr>
  - unbonding_id=<id>
  - regen_amount=<uamt>

4) Rebalance
```json
{ "rebalance": {} }
```
- Admin only; placeholder for delegations rebalance
- Emits:
  - action=rebalance

5) ClaimRewards
```json
{ "claim_rewards": {} }
```
- Admin only; issues DistributionMsg::WithdrawDelegatorReward for each active validator
- Emits:
  - action=claim_rewards
  - claimer=<addr>

6) UpdateValidators
```json
{
  "update_validators": {
    "validators": [
      { "address": "regenvaloper1...", "weight": "0.25" }
    ]
  }
}
```
- Admin only; updates active set and weights (cap by max_validators)
- Emits:
  - action=update_validators
  - count=<active_count>

7) Pause / Resume
```json
{ "pause": {} }
{ "resume": {} }
```
- Admin only; toggles pause_contract
- Emits:
  - action=pause or action=resume

8) UpdateConfig
```json
{
  "update_config": {
    "admin": "regen1new...",
    "fee_rate": "0.04",
    "max_validators": 25
  }
}
```
- Fields optional; fee_rate still capped at 0.20
- Emits:
  - action=update_config

## Query

Route: wasm/query

1) Config
```json
{ "config": {} }
```
Response:
```json
{
  "admin": "regen1...",
  "dregen_token": "regen1cw20...",
  "fee_rate": "0.05",
  "unbonding_period": 1814400,
  "max_validators": 20,
  "min_delegation": "1000000",
  "pause_contract": false
}
```

2) State
```json
{ "state": {} }
```
Response:
```json
{
  "total_regen_staked": "123456",
  "total_dregen_supply": "120000",
  "exchange_rate": "1.0288",
  "last_update_time": 1690001111,
  "total_rewards_claimed": "0",
  "pending_unbonding": "1000"
}
```

3) Exchange Rate
```json
{ "exchange_rate": {} }
```
Response:
```json
{ "rate": "1.0288", "last_updated": 1690001111 }
```

4) Validators
```json
{ "validators": {} }
```
Response:
```json
{ "validators": [ { "address": "regenvaloper1...", "delegated_amount": "0", "weight": "0.2", "last_reward_claim": "1690001111", "slashing_events": 0, "uptime_percentage": "1.0", "commission_rate": "0.1", "is_active": true } ] }
```

5) Unbonding requests by user
```json
{ "unbonding": { "user": "regen1..." } }
```
Response:
```json
{ "requests": [ { "user": "regen1...", "dregen_amount": "1000000", "regen_amount": "1010000", "completion_time": "1690100000", "nft_token_id": "unbond-1" } ] }
```

6) Simulate stake
```json
{ "simulate_stake": { "amount": "1000000" } }
```
Response:
```json
{ "dregen_amount": "990000", "exchange_rate": "1.01", "fee_amount": "50000" }
```

7) Simulate unbond
```json
{ "simulate_unbond": { "dregen_amount": "1000000" } }
```
Response:
```json
{ "regen_amount": "1010000", "completion_time": 1690100000, "fee_amount": "50000" }
```

## Errors

See [contracts/regen-liquid-staking/src/error.rs](../contracts/regen-liquid-staking/src/error.rs).

- Unauthorized
- InvalidFeeRate
- ContractPaused
- InsufficientStake { minimum, received }
- ValidatorNotFound { validator }
- InvalidUnbondAmount
- UnbondingNotComplete { completion_time }

## Events and Indexing

The contract emits standard wasm event attributes on execute:
- action=stake|unbond|claim_unbonding|claim_rewards|rebalance|update_validators|pause|resume|update_config
- Contract-specific metadata as described in Execute section

Reference indexer:
- [indexer/src/index.ts](../indexer/src/index.ts)
- [indexer/src/entities.ts](../indexer/src/entities.ts)
- [indexer/src/handlers/index.ts](../indexer/src/handlers/index.ts)

## CW20 Integration

The contract expects an existing CW20 for dREGEN (config.dregen_token), and uses:
- Mint { recipient, amount } on Stake
- BurnFrom { owner, amount } on Unbond

CW20 interface: https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw20-base
