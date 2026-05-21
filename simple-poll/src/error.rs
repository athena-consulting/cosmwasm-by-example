use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Question is empty or too long")]
    InvalidQuestion {},

    #[error("Poll must have at least two options and no more than the configured limit")]
    InvalidOptionCount {},

    #[error("Option labels must be non-empty, unique, and within the configured length")]
    InvalidOptionLabel {},

    #[error("Poll close height must be in the future")]
    InvalidCloseHeight {},

    #[error("Poll is closed")]
    PollClosed {},

    #[error("Poll is already closed")]
    AlreadyClosed {},

    #[error("Invalid option index")]
    InvalidOptionIndex {},

    #[error("Address has already voted in this poll")]
    AlreadyVoted {},

    #[error("Configuration value is outside the allowed range")]
    InvalidConfig {},
}
