use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Price must be greater than zero")]
    InvalidPrice {},

    #[error("Subscription period must be greater than zero")]
    InvalidPeriod {},

    #[error("Expected at least {expected} {denom}")]
    MissingPayment { expected: Uint128, denom: String },

    #[error("Only the configured native denom can be sent")]
    WrongDenom {},

    #[error("No funds are available to withdraw")]
    NoFunds {},
}
