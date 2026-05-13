use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can perform this action")]
    Unauthorized {},

    #[error("Only the budget manager can claim this budget")]
    NotManager {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Budget label cannot be empty")]
    EmptyLabel {},

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Budget is closed")]
    BudgetClosed {},

    #[error("Budget remaining {remaining} is below requested amount {requested}")]
    BudgetExceeded {
        remaining: Uint128,
        requested: Uint128,
    },

    #[error("Tracked balance {available} is below required amount {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },

    #[error("Unreserved balance {available} is below requested amount {requested}")]
    InsufficientUnreserved {
        available: Uint128,
        requested: Uint128,
    },
}
