use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Reminder already exists")]
    ReminderExists {},

    #[error("Reminder not found")]
    ReminderNotFound {},

    #[error("Reminder is not due yet")]
    NotDue {},

    #[error("Only the creator can cancel this reminder")]
    Unauthorized {},

    #[error("Reminder id and note cannot be empty")]
    EmptyField {},
}
