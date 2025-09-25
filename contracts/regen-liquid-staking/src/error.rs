use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid fee rate")]
    InvalidFeeRate {},

    #[error("Contract is paused")]
    ContractPaused {},

    #[error("Insufficient stake: minimum {minimum}, received {received}")]
    InsufficientStake {
        minimum: Uint128,
        received: Uint128,
    },

    #[error("Validator not found: {validator}")]
    ValidatorNotFound { validator: String },

    #[error("Invalid unbond amount")]
    InvalidUnbondAmount {},

    #[error("Unbonding not complete. Completion time: {completion_time}")]
    UnbondingNotComplete { completion_time: u64 },
}