use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Minimum gap must be at least 1")]
    InvalidMinGap {},

    #[error("Maximum gap must be at least the minimum gap")]
    InvalidMaxGap {},

    #[error("Maximum note length must be between 1 and 500")]
    InvalidMaxNoteLen {},

    #[error("Note is too long")]
    NoteTooLong {},

    #[error("Check-in is too soon; next allowed height is {next_allowed_height}")]
    TooSoon { next_allowed_height: u64 },
}
