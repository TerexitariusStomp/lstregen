use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub distributor: Addr,
    /// Denom for rewards, e.g. "uregen"
    pub reward_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardRecord {
    pub id: u64,
    pub validator: String,
    pub amount: Uint128,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimRecord {
    pub user: Addr,
    pub amount: Uint128,
    pub timestamp: Timestamp,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_REWARD_ID: Item<u64> = Item::new("next_reward_id");
pub const REWARD_HISTORY: Map<u64, RewardRecord> = Map::new("reward_history");
pub const CLAIM_HISTORY: Map<u64, ClaimRecord> = Map::new("claim_history");