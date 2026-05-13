use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the admin can perform this action")]
    Unauthorized {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Deposit must include funds")]
    MissingPayment {},

    #[error("Deposit must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount must be greater than zero")]
    InvalidPaymentAmount {},

    #[error("Distribution batch cannot be empty")]
    EmptyPaymentList {},

    #[error("Duplicate recipient in distribution batch")]
    DuplicateRecipient {},

    #[error("Insufficient contract balance: available {available}, required {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },

    #[error("New admin must be different from the current admin")]
    SameAdmin {},
}
