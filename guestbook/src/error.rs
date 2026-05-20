use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Display name cannot be empty")]
    EmptyDisplayName {},

    #[error("Message cannot be empty")]
    EmptyMessage {},

    #[error("Message is longer than the configured maximum")]
    MessageTooLong {},

    #[error("Maximum message length must be between 1 and 1000")]
    InvalidMaxMessageLen {},
}
