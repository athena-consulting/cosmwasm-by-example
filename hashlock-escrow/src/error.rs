use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Exactly one native coin must be locked at instantiation")]
    SingleCoinRequired {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Escrow is already closed")]
    EscrowClosed {},

    #[error("Escrow has expired")]
    Expired {},

    #[error("Escrow has not expired yet")]
    NotExpired {},

    #[error("Preimage must not be empty")]
    EmptyPreimage {},

    #[error("Preimage hash does not match the escrow hash")]
    InvalidPreimage {},
}
