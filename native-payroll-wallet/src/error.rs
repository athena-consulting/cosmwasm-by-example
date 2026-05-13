use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only the owner can perform this action")]
    Unauthorized {},

    #[error("Denom cannot be empty")]
    EmptyDenom {},

    #[error("Period seconds must be greater than zero")]
    InvalidPeriod {},

    #[error("Salary must be greater than zero")]
    InvalidSalary {},

    #[error("Payment must include funds")]
    MissingPayment {},

    #[error("Payment must include exactly one coin")]
    MultipleDenoms {},

    #[error("Funds must use denom {expected}")]
    WrongDenom { expected: String },

    #[error("Amount must be greater than zero")]
    InvalidAmount {},

    #[error("Employee is not configured")]
    EmployeeNotFound {},

    #[error("Salary is not claimable yet")]
    TooEarly {},

    #[error("Insufficient payroll balance: available {available}, required {required}")]
    InsufficientBalance {
        available: Uint128,
        required: Uint128,
    },
}
