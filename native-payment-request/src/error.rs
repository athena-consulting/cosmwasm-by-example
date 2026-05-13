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

    #[error("invoice amount must be greater than zero")]
    InvalidAmount {},

    #[error("due_at must be in the future")]
    InvalidDueAt {},

    #[error("invoice already exists")]
    DuplicateInvoice {},

    #[error("invoice is already paid")]
    AlreadyPaid {},

    #[error("invoice is canceled")]
    AlreadyCanceled {},

    #[error("invoice expired")]
    InvoiceExpired {},

    #[error("payer is not allowed for this invoice")]
    WrongPayer {},

    #[error("expected exactly {expected} {denom}, got {actual}")]
    InvalidPayment {
        expected: Uint128,
        actual: Uint128,
        denom: String,
    },
}
