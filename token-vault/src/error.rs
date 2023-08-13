use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug,PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Address not whitelisted")]
    NotWhitelisted {},

    #[error("To Do Error")]
    ToDo {},
}
