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

    #[error("Display name is too long")]
    DisplayNameTooLong {},

    #[error("Metadata is too long")]
    MetadataTooLong {},

    #[error("Validation limits must be greater than zero")]
    InvalidLimit {},

    #[error("Score must be higher than the current best score")]
    ScoreNotHighEnough {},

    #[error("Score entry not found")]
    ScoreNotFound {},
}
