use cosmwasm_std::{StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid price")]
    InvalidPrice {},

    #[error("{0}")]
    BidPaymentError(#[from] PaymentError),

    #[error("Incorrect bid payment: expected {0}, actual {1}")]
    IncorrectBidPayment(Uint128, Uint128),

    #[error("Invalid reserve price: reserve_price {0} < starting_price {1}")]
    InvalidReservePrice(Uint128, Uint128),

    #[error("Invalid start / end time: ${0}")]
    InvalidStartEndTime(String),

    #[error("Auction already exists: token_id {0}")]
    AlreadyExists(String),

    #[error("Auction not found: token_id {0}")]
    NotFound(String),

    #[error("Auction invalid status: {0}")]
    InvalidStatus(String),

    #[error("Auction bid too low")]
    BidTooLow {},

    #[error("Reserve price restriction: {0}")]
    ReservePriceRestriction(String),

    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}
