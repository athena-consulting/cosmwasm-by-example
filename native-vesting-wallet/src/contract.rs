#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Timestamp, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StatusResponse};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-vesting-wallet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let denom = msg.denom.trim().to_string();
    if denom.is_empty() {
        return Err(ContractError::EmptyDenom {});
    }
    validate_schedule(msg.start_time, msg.cliff_time, msg.end_time)?;

    let initial_amount = get_sent_amount(&info.funds, &denom);
    if initial_amount.is_zero() {
        return Err(ContractError::NoFunds { denom });
    }

    let recipient = deps.api.addr_validate(&msg.recipient)?;
    let config = Config {
        owner: info.sender,
        recipient,
        denom: denom.clone(),
        start_time: msg.start_time,
        cliff_time: msg.cliff_time.unwrap_or(msg.start_time),
        end_time: msg.end_time,
        total_amount: initial_amount,
        released_amount: Uint128::zero(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("denom", denom)
        .add_attribute("total_amount", initial_amount.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env),
        ExecuteMsg::Deposit {} => execute_deposit(deps, info),
    }
}

pub fn execute_claim(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let status = build_status(&config, env.block.time);

    if status.claimable_amount.is_zero() {
        return Err(ContractError::NoClaimable {});
    }

    config.released_amount += status.claimable_amount;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: config.recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: status.claimable_amount,
            }],
        })
        .add_attribute("action", "claim")
        .add_attribute("recipient", config.recipient)
        .add_attribute("amount", status.claimable_amount.to_string()))
}

pub fn execute_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let amount = get_sent_amount(&info.funds, &config.denom);

    if amount.is_zero() {
        return Err(ContractError::NoFunds {
            denom: config.denom,
        });
    }

    config.total_amount += amount;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("amount", amount.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Status {} => to_binary(&query_status(deps, env)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        recipient: config.recipient.to_string(),
        denom: config.denom,
        start_time: config.start_time,
        cliff_time: config.cliff_time,
        end_time: config.end_time,
        total_amount: config.total_amount,
        released_amount: config.released_amount,
    })
}

pub fn query_status(deps: Deps, env: Env) -> StdResult<StatusResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(build_status(&config, env.block.time))
}

fn validate_schedule(
    start_time: u64,
    cliff_time: Option<u64>,
    end_time: u64,
) -> Result<(), ContractError> {
    if start_time >= end_time {
        return Err(ContractError::InvalidSchedule {});
    }

    if let Some(cliff) = cliff_time {
        if cliff < start_time || cliff > end_time {
            return Err(ContractError::InvalidSchedule {});
        }
    }

    Ok(())
}

fn get_sent_amount(funds: &[Coin], denom: &str) -> Uint128 {
    funds
        .iter()
        .find(|coin| coin.denom == denom)
        .map(|coin| coin.amount)
        .unwrap_or_else(Uint128::zero)
}

fn build_status(config: &Config, now: Timestamp) -> StatusResponse {
    let now = now.seconds();
    let vested_amount = vested_amount(config, now);
    let claimable_amount = vested_amount - config.released_amount;
    let locked_amount = config.total_amount - vested_amount;

    StatusResponse {
        now,
        total_amount: config.total_amount,
        vested_amount,
        released_amount: config.released_amount,
        claimable_amount,
        locked_amount,
    }
}

fn vested_amount(config: &Config, now: u64) -> Uint128 {
    if now < config.cliff_time {
        return Uint128::zero();
    }

    if now >= config.end_time {
        return config.total_amount;
    }

    let elapsed = now.saturating_sub(config.start_time);
    let duration = config.end_time - config.start_time;

    config
        .total_amount
        .multiply_ratio(elapsed as u128, duration as u128)
}
