#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Api, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    CoinShare, ConfigResponse, ExecuteMsg, InstantiateMsg, MemberResponse, PreviewResponse,
    QueryMsg,
};
use crate::state::{Config, Member, CONFIG};

const CONTRACT_NAME: &str = "crates.io:payment-splitter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = validate_config(deps.api, info.sender, msg)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("denom", config.denom)
        .add_attribute("members", config.members.len().to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Distribute {} => execute_distribute(deps, info),
    }
}

pub fn execute_distribute(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let payment = get_sent_amount(&info.funds, &config.denom);

    if payment.is_zero() {
        return Err(ContractError::NoFunds {
            denom: config.denom,
        });
    }

    let preview = build_preview(&config, payment);
    let mut messages: Vec<BankMsg> = vec![];

    for share in preview.shares {
        if !share.amount.is_zero() {
            messages.push(BankMsg::Send {
                to_address: share.address,
                amount: vec![Coin {
                    denom: config.denom.clone(),
                    amount: share.amount,
                }],
            });
        }
    }

    if !preview.remainder.is_zero() {
        messages.push(BankMsg::Send {
            to_address: config.remainder_recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: preview.remainder,
            }],
        });
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "distribute")
        .add_attribute("denom", config.denom)
        .add_attribute("amount", payment.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Preview { amount } => to_binary(&query_preview(deps, amount)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        denom: config.denom,
        members: config
            .members
            .into_iter()
            .map(|member| MemberResponse {
                address: member.address.to_string(),
                share: member.share,
            })
            .collect(),
        total_shares: config.total_shares,
        remainder_recipient: config.remainder_recipient.to_string(),
    })
}

pub fn query_preview(deps: Deps, amount: Uint128) -> StdResult<PreviewResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(build_preview(&config, amount))
}

fn validate_config(
    api: &dyn Api,
    sender: cosmwasm_std::Addr,
    msg: InstantiateMsg,
) -> Result<Config, ContractError> {
    let denom = msg.denom.trim().to_string();
    if denom.is_empty() {
        return Err(ContractError::EmptyDenom {});
    }

    if msg.members.is_empty() {
        return Err(ContractError::NoMembers {});
    }

    let mut total_shares = 0u128;
    let mut members: Vec<Member> = vec![];

    for member in msg.members {
        if member.share == 0 {
            return Err(ContractError::ZeroShare {});
        }

        let address = api.addr_validate(&member.address)?;
        if members.iter().any(|existing| existing.address == address) {
            return Err(ContractError::DuplicateMember {
                address: address.to_string(),
            });
        }

        total_shares = total_shares
            .checked_add(member.share as u128)
            .ok_or(ContractError::ShareOverflow {})?;
        members.push(Member {
            address,
            share: member.share,
        });
    }

    let remainder_recipient = match msg.remainder_recipient {
        Some(address) => api.addr_validate(&address)?,
        None => sender,
    };

    Ok(Config {
        denom,
        members,
        total_shares,
        remainder_recipient,
    })
}

fn get_sent_amount(funds: &[Coin], denom: &str) -> Uint128 {
    funds
        .iter()
        .find(|coin| coin.denom == denom)
        .map(|coin| coin.amount)
        .unwrap_or_else(Uint128::zero)
}

fn build_preview(config: &Config, amount: Uint128) -> PreviewResponse {
    let mut distributed = Uint128::zero();
    let mut shares = vec![];

    for member in &config.members {
        let member_amount = amount.multiply_ratio(member.share as u128, config.total_shares);
        distributed += member_amount;
        shares.push(CoinShare {
            address: member.address.to_string(),
            amount: member_amount,
        });
    }

    PreviewResponse {
        shares,
        remainder: amount - distributed,
        remainder_recipient: config.remainder_recipient.to_string(),
    }
}
