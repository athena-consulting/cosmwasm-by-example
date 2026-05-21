use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Text field cannot be empty")]
    EmptyText {},

    #[error("Text field is too long")]
    TextTooLong {},

    #[error("A quiz needs at least two answer options")]
    NotEnoughOptions {},

    #[error("Too many answer options")]
    TooManyOptions {},

    #[error("Correct option index is out of range")]
    InvalidCorrectOption {},

    #[error("Selected option index is out of range")]
    InvalidSelectedOption {},

    #[error("Validation limits must be greater than zero")]
    InvalidLimit {},

    #[error("Quiz is closed")]
    QuizClosed {},

    #[error("Player already answered this quiz")]
    AlreadyAnswered {},

    #[error("Quiz not found")]
    QuizNotFound {},
}
