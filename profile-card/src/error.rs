use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("display name cannot be empty")]
    EmptyDisplayName {},

    #[error("display name is too long")]
    DisplayNameTooLong {},

    #[error("bio is too long")]
    BioTooLong {},

    #[error("website is too long")]
    WebsiteTooLong {},
}
