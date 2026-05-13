use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can perform this action")]
    Unauthorized {},

    #[error("Only the approved claimant can claim this reimbursement")]
    WrongClaimant {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Reimbursement claim does not exist")]
    ClaimNotFound {},

    #[error("Reimbursement claim has already been paid")]
    ClaimAlreadyPaid {},

    #[error("Insufficient pool balance: available {available}, required {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
