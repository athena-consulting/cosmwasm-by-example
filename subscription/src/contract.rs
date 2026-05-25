#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
    Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SubResponse};
use crate::state::{Config, CONFIG, SUBSCRIPTIONS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        provider: msg.provider,
        price: msg
            .price
            .parse()
            .map_err(|_| ContractError::InvalidPrice {})?,
        denom: msg.denom,
        interval_seconds: msg.interval_seconds,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Subscribe {} => execute_subscribe(deps, env, info),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::Cancel {} => execute_cancel(deps, info),
    }
}

pub fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let coin = info
        .funds
        .iter()
        .find(|c| c.denom == config.denom)
        .ok_or(ContractError::InsufficientFunds {})?;

    if coin.amount.u128() < config.price {
        return Err(ContractError::InsufficientFunds {});
    }

    let next_claim = env.block.time.seconds() + config.interval_seconds;
    SUBSCRIPTIONS.save(deps.storage, info.sender.as_str(), &next_claim)?;

    let msg = BankMsg::Send {
        to_address: config.provider,
        amount: vec![Coin {
            denom: config.denom,
            amount: Uint128::from(config.price),
        }],
    };

    Ok(Response::new()
        .add_attribute("action", "subscribe")
        .add_message(msg))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender.as_str() != config.provider {
        return Err(ContractError::Unauthorized {});
    }

    let now = env.block.time.seconds();
    let mut saw_subscription = false;
    let mut due_subscriber = None;

    for item in SUBSCRIPTIONS.range(deps.storage, None, None, Order::Ascending) {
        let (subscriber, next_claim) = item?;
        saw_subscription = true;

        if next_claim <= now {
            due_subscriber = Some(subscriber);
            break;
        }
    }

    if let Some(subscriber) = due_subscriber {
        let new_next_claim = now + config.interval_seconds;
        SUBSCRIPTIONS.save(deps.storage, subscriber.as_str(), &new_next_claim)?;

        return Ok(Response::new()
            .add_attribute("action", "claim")
            .add_attribute("subscriber", subscriber)
            .add_attribute("next_claim", new_next_claim.to_string()));
    }

    if saw_subscription {
        Err(ContractError::TooEarlyToClaim {})
    } else {
        Err(ContractError::NoSubscription {})
    }
}

pub fn execute_cancel(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    SUBSCRIPTIONS.remove(deps.storage, info.sender.as_str());
    Ok(Response::new().add_attribute("action", "cancel"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Subscription { address } => to_binary(&query_sub(deps, address)?),
    }
}

fn query_sub(deps: Deps, address: String) -> StdResult<SubResponse> {
    let next_claim = SUBSCRIPTIONS.load(deps.storage, &address).unwrap_or(0);
    Ok(SubResponse { next_claim })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            provider: "provider".to_string(),
            price: "100".to_string(),
            denom: "earth".to_string(),
            interval_seconds: 3600,
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn reject_invalid_price() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            provider: "provider".to_string(),
            price: "not-a-number".to_string(),
            denom: "earth".to_string(),
            interval_seconds: 3600,
        };
        let info = mock_info("creator", &[]);

        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert_eq!(err, ContractError::InvalidPrice {});
    }

    #[test]
    fn provider_can_claim_due_subscription() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            provider: "provider".to_string(),
            price: "100".to_string(),
            denom: "earth".to_string(),
            interval_seconds: 3600,
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        let mut env = mock_env();
        env.block.time = cosmwasm_std::Timestamp::from_seconds(100);
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("subscriber", &coins(100, "earth")),
            ExecuteMsg::Subscribe {},
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("provider", &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::TooEarlyToClaim {});

        env.block.time = cosmwasm_std::Timestamp::from_seconds(3700);
        let res = execute(
            deps.as_mut(),
            env,
            mock_info("provider", &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap();

        assert_eq!(res.attributes[0].value, "claim");
        assert_eq!(
            query_sub(deps.as_ref(), "subscriber".to_string())
                .unwrap()
                .next_claim,
            7300
        );
    }
}
