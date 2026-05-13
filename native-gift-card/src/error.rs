use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Gift must include funds")]
    MissingGiftFunds {},

    #[error("Gift must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Gift amount must be greater than zero")]
    InvalidGiftAmount {},

    #[error("Only the gift recipient can claim it")]
    NotRecipient {},

    #[error("Only the gift creator can cancel it")]
    NotCreator {},

    #[error("Gift has already been settled")]
    GiftSettled {},
}
