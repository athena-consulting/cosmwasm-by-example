use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can perform this action")]
    Unauthorized {},

    #[error("Only the current renter can update the message")]
    NotRenter {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Slot price must be greater than zero")]
    InvalidSlotPrice {},

    #[error("Rental duration must be greater than zero")]
    InvalidRentalDuration {},

    #[error("Slot label cannot be empty")]
    EmptyLabel {},

    #[error("Message cannot be empty")]
    EmptyMessage {},

    #[error("Slot is closed")]
    SlotClosed {},

    #[error("Slot is still rented until {paid_until}")]
    SlotNotAvailable { paid_until: u64 },

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
