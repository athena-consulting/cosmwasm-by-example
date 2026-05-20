use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Task title cannot be empty")]
    EmptyTitle {},

    #[error("Task title is too long")]
    TitleTooLong {},

    #[error("Task description is too long")]
    DescriptionTooLong {},

    #[error("Maximum open tasks must be between 1 and 1000")]
    InvalidMaxOpenTasks {},

    #[error("Maximum open tasks reached")]
    MaxOpenTasksReached {},

    #[error("Task not found")]
    TaskNotFound {},
}
