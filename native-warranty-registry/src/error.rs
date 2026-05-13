use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the contract owner can perform this action")]
    Unauthorized {},

    #[error("Only the product owner can perform this action")]
    NotProductOwner {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Registration fee must be greater than zero")]
    InvalidRegistrationFee {},

    #[error("Coverage period must be greater than zero")]
    InvalidCoveragePeriod {},

    #[error("Serial cannot be empty")]
    EmptySerial {},

    #[error("Metadata cannot be empty")]
    EmptyMetadata {},

    #[error("Claim note cannot be empty")]
    EmptyClaimNote {},

    #[error("Product is already registered and still covered")]
    ProductCovered {},

    #[error("Warranty coverage has expired")]
    CoverageExpired {},

    #[error("Claim is already resolved")]
    ClaimAlreadyResolved {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Payment amount {paid} is below required amount {required}")]
    Underpaid { paid: Uint128, required: Uint128 },

    #[error("Withdraw amount must be greater than zero")]
    InvalidWithdrawAmount {},

    #[error("Tracked native balance {available} is below required amount {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
