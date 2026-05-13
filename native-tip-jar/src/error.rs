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

    #[error("Tip must include funds")]
    MissingTip {},

    #[error("Tip must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Insufficient contract balance: available {available}, requested {requested}")]
    InsufficientBalance {
        available: Uint128,
        requested: Uint128,
    },

    #[error("New owner must be different from the current owner")]
    SameOwner {},
}
