use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Waitlist is full")]
    WaitlistFull {},

    #[error("Address is already on the waitlist")]
    AlreadyJoined {},

    #[error("Entry not found")]
    EntryNotFound {},

    #[error("Address is already admitted")]
    AlreadyAdmitted {},

    #[error("Note cannot be empty")]
    EmptyNote {},

    #[error("Note is too long")]
    NoteTooLong {},

    #[error("Limits must be greater than zero")]
    InvalidLimit {},

    #[error("Max entries cannot be lower than the current active entry count")]
    LimitBelowActiveEntries {},
}
