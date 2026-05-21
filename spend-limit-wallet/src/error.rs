use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Allowance not found")]
    AllowanceNotFound {},

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Limit must be greater than zero")]
    InvalidLimit {},

    #[error("Period must be greater than zero")]
    InvalidPeriod {},

    #[error("Spend limit exceeded: requested {requested}, remaining {remaining}")]
    LimitExceeded {
        requested: cosmwasm_std::Uint128,
        remaining: cosmwasm_std::Uint128,
    },
}
