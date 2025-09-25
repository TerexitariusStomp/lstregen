use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use crate::state::{RewardRecord, ClaimRecord};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub distributor: String,
    pub reward_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Set the distributor address allowed to record rewards and trigger distributions
    SetDistributor { distributor: String },
    /// Transfer admin to a new address
    TransferAdmin { new_admin: String },
    /// Record a reward inbound for a validator (called by distributor)
    RecordReward { validator: String, amount: Uint128 },
    /// Record a user claim (accounting only; payouts are done by liquid staking contract)
    RecordClaim { user: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Return current configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Return recent reward history (descending id)
    #[returns(RewardHistoryResponse)]
    RewardHistory {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Return recent claim history (descending index)
    #[returns(ClaimHistoryResponse)]
    ClaimHistory {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub distributor: String,
    pub reward_denom: String,
}

#[cw_serde]
pub struct RewardHistoryResponse {
    pub records: Vec<RewardRecord>,
}

#[cw_serde]
pub struct ClaimHistoryResponse {
    pub records: Vec<ClaimRecord>,
}