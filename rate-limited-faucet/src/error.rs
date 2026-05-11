use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Claim amount must be greater than zero")]
    InvalidClaimAmount {},

    #[error("Denom must not be empty")]
    InvalidDenom {},

    #[error("Cooldown has not expired; next claim is available at {next_claim_at}")]
    CooldownNotExpired { next_claim_at: u64 },
}
