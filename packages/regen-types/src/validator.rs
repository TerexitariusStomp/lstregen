use cosmwasm_std::{Decimal, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorParams {
    pub address: String,
    pub weight: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorInfoView {
    pub address: String,
    pub delegated_amount: Uint128,
    pub weight: Decimal,
    pub last_reward_claim: Timestamp,
    pub slashing_events: u32,
    pub uptime_percentage: Decimal,
    pub commission_rate: Decimal,
    pub is_active: bool,
}