use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Already approved")]
    AlreadyApproved {},

    #[error("Invalid threshold")]
    InvalidThreshold {},

    #[error("Not enough approvals")]
    NotEnoughApprovals {},
}
