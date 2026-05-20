use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("exactly one native coin must fund the vesting wallet")]
    InvalidFunds {},

    #[error("vesting schedule must satisfy start <= cliff < end")]
    InvalidSchedule {},

    #[error("unauthorized")]
    Unauthorized {},

    #[error("no vested funds are currently claimable")]
    NothingToClaim {},

    #[error("not implemented")]
    NotImplemented {},
}
