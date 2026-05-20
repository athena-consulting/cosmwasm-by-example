#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VoucherResponse};
use crate::state::{Voucher, VoucherStatus, VOUCHERS};

const CONTRACT_NAME: &str = "crates.io:native-claim-voucher";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateVoucher {
            id,
            recipient,
            expires_at_height,
        } => execute_create_voucher(deps, info, id, recipient, expires_at_height),
        ExecuteMsg::Claim { id } => execute_claim(deps, env, info, id),
        ExecuteMsg::RefundExpired { id } => execute_refund_expired(deps, env, info, id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Voucher { id } => to_json_binary(&query_voucher(deps, id)?),
    }
}

pub fn execute_create_voucher(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
    recipient: String,
    expires_at_height: u64,
) -> Result<Response, ContractError> {
    if info.funds.is_empty() {
        return Err(ContractError::EmptyVoucher {});
    }

    if VOUCHERS.may_load(deps.storage, &id)?.is_some() {
        return Err(ContractError::VoucherAlreadyExists {});
    }

    let recipient = deps.api.addr_validate(&recipient)?;
    let voucher = Voucher {
        creator: info.sender.clone(),
        recipient: recipient.clone(),
        funds: info.funds,
        expires_at_height,
        status: VoucherStatus::Open,
    };
    VOUCHERS.save(deps.storage, &id, &voucher)?;

    Ok(Response::new()
        .add_attribute("action", "create_voucher")
        .add_attribute("id", id)
        .add_attribute("creator", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("expires_at_height", expires_at_height.to_string()))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    let mut voucher = VOUCHERS.load(deps.storage, &id)?;

    if voucher.status != VoucherStatus::Open {
        return Err(ContractError::VoucherAlreadySettled {});
    }
    if info.sender != voucher.recipient {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.height > voucher.expires_at_height {
        return Err(ContractError::VoucherExpired {});
    }

    voucher.status = VoucherStatus::Claimed;
    VOUCHERS.save(deps.storage, &id, &voucher)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: voucher.recipient.to_string(),
            amount: voucher.funds,
        })
        .add_attribute("action", "claim")
        .add_attribute("id", id)
        .add_attribute("recipient", info.sender))
}

pub fn execute_refund_expired(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    let mut voucher = VOUCHERS.load(deps.storage, &id)?;

    if voucher.status != VoucherStatus::Open {
        return Err(ContractError::VoucherAlreadySettled {});
    }
    if info.sender != voucher.creator {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.height <= voucher.expires_at_height {
        return Err(ContractError::VoucherNotExpired {});
    }

    voucher.status = VoucherStatus::Refunded;
    VOUCHERS.save(deps.storage, &id, &voucher)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: voucher.creator.to_string(),
            amount: voucher.funds,
        })
        .add_attribute("action", "refund_expired")
        .add_attribute("id", id)
        .add_attribute("creator", info.sender))
}

pub fn query_voucher(deps: Deps, id: String) -> StdResult<VoucherResponse> {
    let voucher = VOUCHERS.load(deps.storage, &id)?;

    Ok(VoucherResponse {
        id,
        creator: voucher.creator.to_string(),
        recipient: voucher.recipient.to_string(),
        funds: voucher.funds,
        expires_at_height: voucher.expires_at_height,
        status: status_label(&voucher.status).to_string(),
    })
}

fn status_label(status: &VoucherStatus) -> &'static str {
    match status {
        VoucherStatus::Open => "open",
        VoucherStatus::Claimed => "claimed",
        VoucherStatus::Refunded => "refunded",
    }
}
