use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Limits must be greater than zero")]
    InvalidLimit {},

    #[error("{field} cannot be empty")]
    InvalidText { field: String },

    #[error("{field} is too long")]
    TextTooLong { field: String },

    #[error("Course must include at least one lesson")]
    NoLessons {},

    #[error("Course has too many lessons")]
    TooManyLessons {},

    #[error("Course not found")]
    CourseNotFound {},

    #[error("Course is archived")]
    CourseArchived {},

    #[error("Lesson index is out of range")]
    InvalidLessonIndex {},

    #[error("Lesson already completed")]
    LessonAlreadyCompleted {},

    #[error("Progress not found")]
    ProgressNotFound {},
}
