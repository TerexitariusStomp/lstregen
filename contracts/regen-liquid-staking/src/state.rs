use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub dregen_token: Addr,
    pub fee_rate: Decimal,
    pub unbonding_period: u64,
    pub max_validators: u32,
    pub min_delegation: Uint128,
    pub pause_contract: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_regen_staked: Uint128,
    pub total_dregen_supply: Uint128,
    pub exchange_rate: Decimal,
    pub last_update_time: Timestamp,
    pub total_rewards_claimed: Uint128,
    pub pending_unbonding: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorInfo {
    pub address: String,
    pub delegated_amount: Uint128,
    pub weight: Decimal,
    pub last_reward_claim: Timestamp,
    pub slashing_events: u32,
    pub uptime_percentage: Decimal,
    pub commission_rate: Decimal,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnbondingRequest {
    pub user: Addr,
    pub dregen_amount: Uint128,
    pub regen_amount: Uint128,
    pub completion_time: Timestamp,
    pub nft_token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardDistribution {
    pub validator: String,
    pub amount: Uint128,
    pub distribution_time: Timestamp,
}

// Storage items
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const VALIDATORS: Map<String, ValidatorInfo> = Map::new("validators");
pub const UNBONDING_REQUESTS: Map<u64, UnbondingRequest> = Map::new("unbonding");
pub const USER_UNBONDING: Map<(&Addr, u64), bool> = Map::new("user_unbonding");
pub const REWARD_HISTORY: Map<u64, RewardDistribution> = Map::new("rewards");
pub const NEXT_UNBONDING_ID: Item<u64> = Item::new("next_unbonding_id");