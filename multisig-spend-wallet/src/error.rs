use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("owner set cannot be empty")]
    EmptyOwners {},

    #[error("duplicate owner")]
    DuplicateOwner {},

    #[error("threshold must be between one and owner count")]
    InvalidThreshold {},

    #[error("unauthorized")]
    Unauthorized {},

    #[error("spend amount must be greater than zero")]
    InvalidAmount {},

    #[error("proposal already executed")]
    AlreadyExecuted {},

    #[error("owner already approved this proposal")]
    AlreadyApproved {},

    #[error("not implemented")]
    NotImplemented {},
}
