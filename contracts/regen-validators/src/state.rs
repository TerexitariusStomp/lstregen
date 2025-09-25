use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
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

pub const CONFIG: Item<Config> = Item::new("config");
pub const VALIDATORS: Map<String, ValidatorInfo> = Map::new("validators");