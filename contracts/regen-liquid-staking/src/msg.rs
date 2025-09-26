use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal, Uint128};

use crate::state::{UnbondingRequest, ValidatorInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub fee_rate: Decimal,
    pub unbonding_period: u64,
    pub max_validators: u32,
    pub min_delegation: Uint128,
    /// Optional at instantiate. If omitted, you must set it later via UpdateConfig.
    pub dregen_token: Option<String>,
    pub validators: Vec<ValidatorParams>,
}

#[cw_serde]
pub struct ValidatorParams {
    pub address: String,
    pub weight: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Stake REGEN tokens and mint dREGEN
    Stake {},
    /// Initiate unbonding process
    Unbond { dregen_amount: Uint128 },
    /// Claim completed unbonding
    ClaimUnbonding { unbonding_id: u64 },
    /// Rebalance delegations across validators
    Rebalance {},
    /// Claim rewards from all validators
    ClaimRewards {},
    /// Update validator set
    UpdateValidators { validators: Vec<ValidatorParams> },
    /// Emergency pause contract
    Pause {},
    /// Resume contract operations
    Resume {},
    /// Update configuration
    UpdateConfig {
        admin: Option<String>,
        fee_rate: Option<Decimal>,
        max_validators: Option<u32>,
        dregen_token: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get current configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Get current state
    #[returns(StateResponse)]
    State {},
    /// Get exchange rate between REGEN and dREGEN
    #[returns(ExchangeRateResponse)]
    ExchangeRate {},
    /// Get validator information
    #[returns(ValidatorsResponse)]
    Validators {},
    /// Get user's unbonding requests
    #[returns(UnbondingResponse)]
    Unbonding { user: String },
    /// Simulate staking operation
    #[returns(SimulateStakeResponse)]
    SimulateStake { amount: Uint128 },
    /// Simulate unbonding operation
    #[returns(SimulateUnbondResponse)]
    SimulateUnbond { dregen_amount: Uint128 },
}

// Response types
#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub dregen_token: String,
    pub fee_rate: Decimal,
    pub unbonding_period: u64,
    pub max_validators: u32,
    pub min_delegation: Uint128,
    pub pause_contract: bool,
}

#[cw_serde]
pub struct StateResponse {
    pub total_regen_staked: Uint128,
    pub total_dregen_supply: Uint128,
    pub exchange_rate: Decimal,
    pub last_update_time: u64,
    pub total_rewards_claimed: Uint128,
    pub pending_unbonding: Uint128,
}

#[cw_serde]
pub struct ExchangeRateResponse {
    pub rate: Decimal,
    pub last_updated: u64,
}

#[cw_serde]
pub struct ValidatorsResponse {
    pub validators: Vec<ValidatorInfo>,
}

#[cw_serde]
pub struct UnbondingResponse {
    pub requests: Vec<UnbondingRequest>,
}

#[cw_serde]
pub struct SimulateStakeResponse {
    pub dregen_amount: Uint128,
    pub exchange_rate: Decimal,
    pub fee_amount: Uint128,
}

#[cw_serde]
pub struct SimulateUnbondResponse {
    pub regen_amount: Uint128,
    pub completion_time: u64,
    pub fee_amount: Uint128,
}