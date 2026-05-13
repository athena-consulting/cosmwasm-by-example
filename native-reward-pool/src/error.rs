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

    #[error("Deposit must include funds")]
    MissingPayment {},

    #[error("Deposit must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Allocated rewards exceed pool balance: balance {balance}, allocated {allocated}")]
    InsufficientPool {
        balance: Uint128,
        allocated: Uint128,
    },

    #[error("No reward is available for this address")]
    NoReward {},

    #[error("New owner must be different from the current owner")]
    SameOwner {},
}
