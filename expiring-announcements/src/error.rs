use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Announcement already exists")]
    AnnouncementExists {},

    #[error("Announcement not found")]
    AnnouncementNotFound {},

    #[error("Announcement id, title, and body cannot be empty")]
    EmptyField {},

    #[error("Expiration height must be in the future")]
    InvalidExpiration {},
}
