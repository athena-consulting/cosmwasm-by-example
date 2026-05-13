use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Unlock time must be in the future")]
    UnlockNotFuture {},

    #[error("Lock does not exist")]
    LockNotFound {},

    #[error("Only the lock owner can perform this action")]
    Unauthorized {},

    #[error("Lock is already claimed")]
    AlreadyClaimed {},

    #[error("Lock has not reached unlock time")]
    TooEarly {},
}
