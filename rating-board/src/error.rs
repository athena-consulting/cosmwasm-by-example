use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Item name cannot be empty")]
    EmptyName {},

    #[error("Item name is too long")]
    NameTooLong {},

    #[error("Item description is too long")]
    DescriptionTooLong {},

    #[error("Maximum rating must be between 1 and 100")]
    InvalidMaxRating {},

    #[error("Rating must be between 1 and the configured maximum")]
    InvalidRating {},

    #[error("Item not found")]
    ItemNotFound {},
}
