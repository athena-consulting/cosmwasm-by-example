use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can perform this action")]
    Unauthorized {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("At least one royalty share is required")]
    EmptyShares {},

    #[error("Royalty share bps must be greater than zero")]
    InvalidShareBps {},

    #[error("Royalty shares must sum to 10000 bps")]
    InvalidShareTotal {},

    #[error("Duplicate royalty recipient")]
    DuplicateRecipient {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount must be greater than zero")]
    InvalidPaymentAmount {},

    #[error("Payment is too small to split across the configured recipients")]
    PaymentTooSmall {},

    #[error("New owner must be different from the current owner")]
    SameOwner {},
}
