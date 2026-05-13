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

    #[error("Seat price must be greater than zero")]
    InvalidSeatPrice {},

    #[error("Maximum seats must be greater than zero")]
    InvalidMaxSeats {},

    #[error("All seats have been sold")]
    SoldOut {},

    #[error("Account already owns a seat")]
    SeatAlreadyOwned {},

    #[error("Sender does not own a seat")]
    SeatNotOwned {},

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
