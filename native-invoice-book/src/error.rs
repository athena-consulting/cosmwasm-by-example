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

    #[error("Invoice amount must be greater than zero")]
    InvalidInvoiceAmount {},

    #[error("Invoice does not exist")]
    InvoiceNotFound {},

    #[error("Invoice is already paid")]
    InvoiceAlreadyPaid {},

    #[error("Only the configured payer can pay this invoice")]
    WrongPayer {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount {paid} is below required invoice amount {required}")]
    Underpaid { paid: Uint128, required: Uint128 },

    #[error("Withdraw amount must be greater than zero")]
    InvalidWithdrawAmount {},

    #[error("Insufficient collected balance: available {available}, required {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
