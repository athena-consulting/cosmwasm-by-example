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

    #[error("Item name cannot be empty")]
    EmptyItem {},

    #[error("Unit price must be greater than zero")]
    InvalidUnitPrice {},

    #[error("Quantity must be greater than zero")]
    InvalidQuantity {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount {paid} is below required amount {required}")]
    Underpaid { paid: Uint128, required: Uint128 },

    #[error("Preorder max units would be exceeded")]
    SoldOut {},

    #[error("Order does not exist")]
    OrderNotFound {},

    #[error("Order is already fulfilled")]
    AlreadyFulfilled {},

    #[error("Order is already refunded")]
    AlreadyRefunded {},

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Insufficient tracked balance: available {available}, required {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
