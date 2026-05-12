use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("denom cannot be empty")]
    EmptyDenom {},

    #[error("vesting schedule is invalid")]
    InvalidSchedule {},

    #[error("no funds were sent for denom {denom}")]
    NoFunds { denom: String },

    #[error("no vested funds are claimable")]
    NoClaimable {},
}
