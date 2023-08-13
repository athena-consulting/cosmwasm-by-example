use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("sent fund should be 2 abc")]
    InsufficientFunds(),

    #[error(
        "there can't be more than 1 transaction originating from this contract at a given time"
    )]
    BlockTimestampError(),
}
