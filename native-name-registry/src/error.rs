use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the contract owner can perform this action")]
    Unauthorized {},

    #[error("Only the current name owner can perform this action")]
    NotNameOwner {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Registration fee must be greater than zero")]
    InvalidFee {},

    #[error("Registration period must be greater than zero")]
    InvalidRegistrationPeriod {},

    #[error("Name cannot be empty")]
    EmptyName {},

    #[error("Name value cannot be empty")]
    EmptyValue {},

    #[error("Name is already registered and not expired")]
    NameUnavailable {},

    #[error("Name is expired")]
    NameExpired {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount {paid} is below required amount {required}")]
    Underpaid { paid: Uint128, required: Uint128 },

    #[error("Withdraw amount must be greater than zero")]
    InvalidWithdrawAmount {},

    #[error("Tracked native balance {available} is below required amount {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
