use cosmwasm_std::{Addr, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardRecordView {
    pub id: u64,
    pub validator: String,
    pub amount: Uint128,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimRecordView {
    pub user: Addr,
    pub amount: Uint128,
    pub timestamp: Timestamp,
}