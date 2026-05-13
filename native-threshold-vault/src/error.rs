use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("At least one member is required")]
    NoMembers {},

    #[error("Member list contains a duplicate address")]
    DuplicateMember {},

    #[error("Threshold must be greater than zero and no larger than member count")]
    InvalidThreshold {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Only vault members can perform this action")]
    Unauthorized {},

    #[error("Sender must attach one positive coin with the configured denom")]
    InvalidDeposit {},

    #[error("Spend amount must be greater than zero")]
    ZeroAmount {},

    #[error("Proposal not found")]
    ProposalNotFound {},

    #[error("Proposal already has this approval")]
    AlreadyApproved {},

    #[error("Proposal has already been executed")]
    AlreadyExecuted {},

    #[error("Proposal does not have enough approvals")]
    ThresholdNotMet {},

    #[error("Vault does not have enough funds for this spend")]
    InsufficientFunds {},
}
