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

    #[error("Deposit amount must be greater than zero")]
    InvalidDeposit {},

    #[error("RSVP window is closed")]
    EventClosed {},

    #[error("Attendee has already reserved")]
    AlreadyReserved {},

    #[error("Attendee has not reserved")]
    NotReserved {},

    #[error("Deposit has already been settled")]
    AlreadySettled {},

    #[error("Attendee is not checked in")]
    NotCheckedIn {},

    #[error("Forfeit is only available after the event time")]
    TooEarlyForForfeit {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Expected deposit {expected}, received {received}")]
    WrongDeposit {
        expected: Uint128,
        received: Uint128,
    },
}
