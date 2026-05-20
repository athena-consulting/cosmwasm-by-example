use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contact already exists")]
    ContactExists {},

    #[error("Contact name cannot be empty")]
    EmptyName {},

    #[error("Contact note cannot be longer than 280 characters")]
    NoteTooLong {},
}
