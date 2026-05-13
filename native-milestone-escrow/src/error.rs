use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("denom cannot be empty")]
    EmptyDenom {},

    #[error("escrow amount must be greater than zero")]
    InvalidAmount {},

    #[error("timeout must be in the future")]
    InvalidTimeout {},

    #[error("escrow already exists")]
    DuplicateEscrow {},

    #[error("escrow is already settled")]
    AlreadySettled {},

    #[error("an arbiter is required when no default arbiter is configured")]
    MissingArbiter {},

    #[error("expected exactly {expected} {denom}, got {actual}")]
    InvalidPayment {
        expected: Uint128,
        actual: Uint128,
        denom: String,
    },
}
