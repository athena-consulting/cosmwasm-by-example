use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Deposit amount must be greater than zero")]
    ZeroDeposit {},

    #[error("Sender must attach exactly the configured deposit coin")]
    InvalidDeposit {},

    #[error("Sender already has a locked deposit")]
    AlreadyDeposited {},

    #[error("No deposit found for this account")]
    NoDeposit {},

    #[error("Refund period has closed")]
    RefundClosed {},

    #[error("Deposit cannot be forfeited before the refund deadline")]
    DeadlineNotReached {},
}
