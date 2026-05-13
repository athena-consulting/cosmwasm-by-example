use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can perform this action")]
    Unauthorized {},

    #[error("Only the beneficiary can release the goal")]
    NotBeneficiary {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Target amount must be greater than zero")]
    InvalidTarget {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Savings goal has already been released")]
    AlreadyReleased {},

    #[error("Savings goal target has not been reached: saved {saved}, target {target}")]
    TargetNotReached { saved: Uint128, target: Uint128 },
}
