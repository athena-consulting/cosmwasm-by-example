use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("limit must be greater than zero")]
    InvalidLimit {},

    #[error("window size must be greater than zero")]
    InvalidWindow {},

    #[error("exactly one native coin in the configured denom is required")]
    InvalidFunds {},

    #[error("spend amount must be greater than zero")]
    InvalidAmount {},

    #[error("spend limit exceeded")]
    SpendLimitExceeded {},

    #[error("not implemented")]
    NotImplemented {},
}
