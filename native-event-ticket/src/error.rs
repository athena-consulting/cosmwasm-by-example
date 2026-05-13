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

    #[error("Ticket price must be greater than zero")]
    InvalidPrice {},

    #[error("Ticket quantity must be greater than zero")]
    InvalidQuantity {},

    #[error("No tickets remain for this purchase")]
    SoldOut {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Expected payment {expected}, received {received}")]
    PaymentMismatch {
        expected: Uint128,
        received: Uint128,
    },

    #[error("Insufficient revenue: available {available}, requested {requested}")]
    InsufficientRevenue {
        available: Uint128,
        requested: Uint128,
    },
}
