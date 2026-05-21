use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid denom")]
    InvalidDenom {},

    #[error("Heartbeat window must be greater than zero")]
    InvalidHeartbeatWindow {},

    #[error("Deposit must include native funds in denom {denom}")]
    MissingDeposit { denom: String },

    #[error("Vault has not expired")]
    NotExpired {},

    #[error("Amount must be greater than zero")]
    ZeroAmount {},

    #[error("Insufficient balance: available {available}, requested {amount}")]
    InsufficientBalance { available: Uint128, amount: Uint128 },
}
