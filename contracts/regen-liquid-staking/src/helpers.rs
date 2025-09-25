use cosmwasm_std::{Addr, MessageInfo, Uint128};

use crate::error::ContractError;
use crate::state::Config;

pub const DENOM_REGEN: &str = "uregen";

/// Extract the amount of uregen sent with the message
pub fn extract_uregen_amount(info: &MessageInfo) -> Uint128 {
    info.funds
        .iter()
        .find(|c| c.denom == DENOM_REGEN)
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero)
}

/// Ensure the contract is not paused
pub fn ensure_not_paused(config: &Config) -> Result<(), ContractError> {
    if config.pause_contract {
        return Err(ContractError::ContractPaused {});
    }
    Ok(())
}

/// Ensure the sender is admin
pub fn ensure_admin(sender: &Addr, config: &Config) -> Result<(), ContractError> {
    if &config.admin != sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}