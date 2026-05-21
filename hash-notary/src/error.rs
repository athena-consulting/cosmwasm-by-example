use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Record not found")]
    RecordNotFound {},

    #[error("Digest already registered")]
    DuplicateDigest {},

    #[error("Digest must be a SHA-256 hex string")]
    InvalidDigest {},

    #[error("Label must be non-empty and at most 80 characters")]
    InvalidLabel {},

    #[error("Memo must be at most 240 characters")]
    InvalidMemo {},
}
