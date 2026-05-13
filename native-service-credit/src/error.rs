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

    #[error("Unit price must be greater than zero")]
    InvalidUnitPrice {},

    #[error("Credit amount must be greater than zero")]
    InvalidCredits {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount {paid} is below required amount {required}")]
    Underpaid { paid: Uint128, required: Uint128 },

    #[error("Insufficient credits: available {available}, required {required}")]
    InsufficientCredits { available: u64, required: u64 },

    #[error("Withdraw amount must be greater than zero")]
    InvalidWithdrawAmount {},

    #[error("Insufficient tracked native balance: available {available}, required {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
