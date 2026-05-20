#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, WindowResponse};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:limited-spend-wallet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.limit.is_zero() {
        return Err(ContractError::InvalidLimit {});
    }
    if msg.window_size == 0 {
        return Err(ContractError::InvalidWindow {});
    }
    if msg.denom.trim().is_empty() {
        return Err(ContractError::InvalidFunds {});
    }

    let spender = deps.api.addr_validate(&msg.spender)?;
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender.clone(),
            spender: spender.clone(),
            denom: msg.denom.clone(),
            limit: msg.limit,
            window_size: msg.window_size,
            window_start: env.block.height,
            spent: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("spender", spender)
        .add_attribute("denom", msg.denom)
        .add_attribute("limit", msg.limit))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => execute_deposit(deps, info),
        ExecuteMsg::Spend { recipient, amount } => {
            execute_spend(deps, env, info, recipient, amount)
        }
        ExecuteMsg::UpdateSpender { spender } => execute_update_spender(deps, info, spender),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Window {} => to_json_binary(&query_window(deps)?),
    }
}

fn execute_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if info.funds.len() != 1
        || info.funds[0].denom != config.denom
        || info.funds[0].amount.is_zero()
    {
        return Err(ContractError::InvalidFunds {});
    }

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("owner", info.sender)
        .add_attribute("amount", info.funds[0].amount))
}

fn execute_spend(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.spender {
        return Err(ContractError::Unauthorized {});
    }

    refresh_window(&mut config, env.block.height);
    let new_spent = config.spent + amount;
    if new_spent > config.limit {
        return Err(ContractError::SpendLimitExceeded {});
    }

    let recipient = deps.api.addr_validate(&recipient)?;
    config.spent = new_spent;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom,
                amount,
            }],
        })
        .add_attribute("action", "spend")
        .add_attribute("spender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount))
}

fn execute_update_spender(
    deps: DepsMut,
    info: MessageInfo,
    spender: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let spender = deps.api.addr_validate(&spender)?;
    config.spender = spender.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_spender")
        .add_attribute("spender", spender))
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        spender: config.spender.to_string(),
        denom: config.denom,
        limit: config.limit,
        window_size: config.window_size,
    })
}

fn query_window(deps: Deps) -> StdResult<WindowResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(WindowResponse {
        window_start: config.window_start,
        spent: config.spent,
        remaining: remaining(&config),
    })
}

fn refresh_window(config: &mut Config, height: u64) {
    if height >= config.window_start.saturating_add(config.window_size) {
        config.window_start = height;
        config.spent = Uint128::zero();
    }
}

fn remaining(config: &Config) -> Uint128 {
    if config.spent >= config.limit {
        Uint128::zero()
    } else {
        config.limit - config.spent
    }
}
