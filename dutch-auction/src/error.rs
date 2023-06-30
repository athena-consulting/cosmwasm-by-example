use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Auction has Expired")]
    AuctionExpired(),

    #[error("The denom is invalid for buying")]
    InvalidDenomination(),

    #[error("Insufficient funds, expected: `{expected}`, actual: `{actual}`!")]
    InsufficientFunds { expected: Uint128, actual: Uint128 },
}
