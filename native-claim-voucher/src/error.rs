use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("voucher already exists")]
    VoucherAlreadyExists {},

    #[error("voucher has no funds")]
    EmptyVoucher {},

    #[error("voucher has expired")]
    VoucherExpired {},

    #[error("voucher has not expired")]
    VoucherNotExpired {},

    #[error("voucher already settled")]
    VoucherAlreadySettled {},
}
