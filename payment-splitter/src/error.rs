use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("denom cannot be empty")]
    EmptyDenom {},

    #[error("at least one member is required")]
    NoMembers {},

    #[error("member share must be greater than zero")]
    ZeroShare {},

    #[error("duplicate member address: {address}")]
    DuplicateMember { address: String },

    #[error("share total overflowed")]
    ShareOverflow {},

    #[error("no funds were sent for denom {denom}")]
    NoFunds { denom: String },
}
