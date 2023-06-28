use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("Numbers can not be zero")]
    CanNotBeZero(),
}
