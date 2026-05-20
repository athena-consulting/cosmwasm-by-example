use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("question must not be empty")]
    EmptyQuestion {},

    #[error("closing height must be in the future")]
    InvalidCloseHeight {},

    #[error("poll is closed")]
    PollClosed {},

    #[error("address has already voted")]
    AlreadyVoted {},

    #[error("not implemented")]
    NotImplemented {},
}
