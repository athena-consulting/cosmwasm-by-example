#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, RecipientShare, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-payment-splitter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.recipients.is_empty() {
        return Err(ContractError::NoRecipients {});
    }

    let mut recipients = Vec::with_capacity(msg.recipients.len());
    let mut total_shares = Uint128::zero();

    for recipient in msg.recipients {
        if recipient.share.is_zero() {
            return Err(ContractError::ZeroShare {});
        }

        let address = deps.api.addr_validate(&recipient.address)?;
        total_shares += recipient.share;
        recipients.push(RecipientShare {
            address,
            share: recipient.share,
        });
    }

    CONFIG.save(
        deps.storage,
        &Config {
            recipients,
            total_shares,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("total_shares", total_shares))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

fn execute_distribute(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    if info.funds.len() != 1 || info.funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFunds {});
    }

    let config = CONFIG.load(deps.storage)?;
    let coin = info.funds[0].clone();
    let mut amounts = Vec::with_capacity(config.recipients.len());
    let mut remainder = coin.amount;

    for recipient in &config.recipients {
        let amount = coin
            .amount
            .multiply_ratio(recipient.share, config.total_shares);
        remainder -= amount;
        amounts.push(amount);
    }

    amounts[0] += remainder;

    let mut response = Response::new()
        .add_attribute("action", "distribute")
        .add_attribute("denom", coin.denom.clone())
        .add_attribute("amount", coin.amount);

    for (recipient, amount) in config.recipients.iter().zip(amounts) {
        if amount.is_zero() {
            continue;
        }

        response = response.add_message(BankMsg::Send {
            to_address: recipient.address.to_string(),
            amount: vec![cosmwasm_std::Coin {
                denom: coin.denom.clone(),
                amount,
            }],
        });
    }

    Ok(response)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let recipients = config
        .recipients
        .into_iter()
        .map(|recipient| crate::msg::RecipientShareMsg {
            address: recipient.address.to_string(),
            share: recipient.share,
        })
        .collect();

    Ok(ConfigResponse {
        recipients,
        total_shares: config.total_shares,
    })
}
