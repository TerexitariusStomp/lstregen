use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Uint128};

use crate::state::ValidatorInfo;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub validators: Vec<ValidatorParams>,
}

#[cw_serde]
pub struct ValidatorParams {
    pub address: String,
    pub weight: Decimal,
    pub commission_rate: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddValidator { params: ValidatorParams },
    RemoveValidator { address: String },
    UpdateWeight { address: String, weight: Decimal },
    SetCommission { address: String, commission_rate: Decimal },
    Activate { address: String },
    Deactivate { address: String },
    TransferAdmin { new_admin: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ValidatorsResponse)]
    Validators {},
    #[returns(ValidatorResponse)]
    Validator { address: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub total_active: u32,
}

#[cw_serde]
pub struct ValidatorsResponse {
    pub validators: Vec<ValidatorInfo>,
}

#[cw_serde]
pub struct ValidatorResponse {
    pub validator: Option<ValidatorInfo>,
}