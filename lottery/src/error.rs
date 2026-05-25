use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("No players")]
    NoPlayers {},

    #[error("Invalid ticket price")]
    InvalidTicketPrice {},

    #[error("Winner already drawn")]
    WinnerAlreadyDrawn {},
}
