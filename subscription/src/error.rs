use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Invalid price")]
    InvalidPrice {},

    #[error("Too early to claim")]
    TooEarlyToClaim {},

    #[error("No subscription")]
    NoSubscription {},

    #[error("Unauthorized")]
    Unauthorized {},
}
