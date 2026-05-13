use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can perform this action")]
    Unauthorized {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Match bps must be between 1 and 10000")]
    InvalidMatchBps {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Donation is too small to produce a nonzero match")]
    MatchTooSmall {},

    #[error("Insufficient match pool: available {available}, required {required}")]
    InsufficientMatchPool {
        available: Uint128,
        required: Uint128,
    },
}
