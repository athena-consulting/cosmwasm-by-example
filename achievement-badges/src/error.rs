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

    #[error("Badge already exists")]
    BadgeExists {},

    #[error("Badge not found")]
    BadgeNotFound {},

    #[error("Badge is archived")]
    BadgeArchived {},

    #[error("Award already exists")]
    AwardExists {},

    #[error("Award not found")]
    AwardNotFound {},

    #[error("Badge limit reached for holder")]
    LimitReached {},

    #[error("The owner must remain an issuer")]
    CannotRemoveOwnerIssuer {},
}
