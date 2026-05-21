use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },

    #[error("Inventory item not found")]
    ItemNotFound {},

    #[error("Inventory item limit reached")]
    ItemLimitReached {},

    #[error("Insufficient quantity")]
    InsufficientQuantity {},
}
