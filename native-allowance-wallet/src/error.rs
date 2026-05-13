use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("denom cannot be empty")]
    EmptyDenom {},

    #[error("period_seconds must be greater than zero")]
    InvalidPeriod {},

    #[error("period_limit must be greater than zero")]
    InvalidLimit {},

    #[error("amount must be greater than zero")]
    InvalidAmount {},

    #[error("expected native token deposit in {denom}, got {actual}")]
    InvalidDeposit { denom: String, actual: Uint128 },

    #[error(
        "withdrawal exceeds current period allowance: remaining {remaining}, requested {requested}"
    )]
    AllowanceExceeded {
        remaining: Uint128,
        requested: Uint128,
    },

    #[error("contract balance too low: available {available}, requested {requested}")]
    InsufficientBalance {
        available: Uint128,
        requested: Uint128,
    },
}
