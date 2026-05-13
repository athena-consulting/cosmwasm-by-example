use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the bill creator can perform this action")]
    Unauthorized {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("A bill needs at least one participant")]
    EmptyParticipants {},

    #[error("A participant appears more than once")]
    DuplicateParticipant {},

    #[error("Share amount must be greater than zero")]
    InvalidShareAmount {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount {paid} must equal share amount {required}")]
    IncorrectPayment { paid: Uint128, required: Uint128 },

    #[error("Bill is not open")]
    BillNotOpen {},

    #[error("Bill is not fully paid")]
    BillNotFullyPaid {},

    #[error("Sender is not a participant on this bill")]
    NotParticipant {},

    #[error("Participant already paid this bill")]
    AlreadyPaid {},

    #[error("Tracked balance {available} is below required amount {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
