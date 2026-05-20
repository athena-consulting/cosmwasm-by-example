use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("at least one recipient is required")]
    NoRecipients {},

    #[error("each recipient must have a non-zero share")]
    ZeroShare {},

    #[error("exactly one native coin must be sent")]
    InvalidFunds {},

    #[error("not implemented")]
    NotImplemented {},
}
