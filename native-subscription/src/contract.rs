#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, IsActiveResponse, QueryMsg, SubscriptionResponse,
    TreasuryResponse,
};
use crate::state::{Config, Subscription, CONFIG, SUBSCRIPTIONS};

const CONTRACT_NAME: &str = "crates.io:native-subscription";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_plan(msg.price, msg.period_seconds)?;

    let admin = match msg.admin {
        Some(admin) => deps.api.addr_validate(&admin)?,
        None => info.sender.clone(),
    };

    let config = Config {
        admin,
        denom: msg.denom,
        price: msg.price,
        period_seconds: msg.period_seconds,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin)
        .add_attribute("denom", config.denom)
        .add_attribute("price", config.price)
        .add_attribute("period_seconds", config.period_seconds.to_string()))
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
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, env, info, recipient, amount)
        }
        ExecuteMsg::UpdateConfig {
            price,
            period_seconds,
            admin,
        } => execute_update_config(deps, info, price, period_seconds, admin),
    }
}

fn execute_subscribe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let paid = paid_amount(&info, &config.denom)?;
    if paid < config.price {
        return Err(ContractError::MissingPayment {
            expected: config.price,
            denom: config.denom,
        });
    }

    let now = env.block.time.seconds();
    let existing = SUBSCRIPTIONS.may_load(deps.storage, &info.sender)?;
    let starts_at = existing
        .map(|subscription| subscription.expires_at)
        .unwrap_or(now)
        .max(now);
    let expires_at = starts_at + config.period_seconds;

    SUBSCRIPTIONS.save(deps.storage, &info.sender, &Subscription { expires_at })?;

    Ok(Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("subscriber", info.sender)
        .add_attribute("paid", paid)
        .add_attribute("denom", config.denom)
        .add_attribute("expires_at", expires_at.to_string()))
}

fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&info, &config)?;

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    let amount = amount.unwrap_or(balance.amount);
    if amount.is_zero() {
        return Err(ContractError::NoFunds {});
    }

    let recipient = match recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => config.admin,
    };

    let msg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw")
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount)
        .add_attribute("denom", config.denom))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    price: Option<Uint128>,
    period_seconds: Option<u64>,
    admin: Option<String>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        assert_admin(&info, &config)?;

        if let Some(price) = price {
            if price.is_zero() {
                return Err(ContractError::InvalidPrice {});
            }
            config.price = price;
        }

        if let Some(period_seconds) = period_seconds {
            if period_seconds == 0 {
                return Err(ContractError::InvalidPeriod {});
            }
            config.period_seconds = period_seconds;
        }

        if let Some(admin) = admin {
            config.admin = deps.api.addr_validate(&admin)?;
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Subscription { subscriber } => {
            to_json_binary(&query_subscription(deps, env, subscriber)?)
        }
        QueryMsg::IsActive { subscriber } => {
            to_json_binary(&query_is_active(deps, env, subscriber)?)
        }
        QueryMsg::Treasury {} => to_json_binary(&query_treasury(deps, env)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin,
        denom: config.denom,
        price: config.price,
        period_seconds: config.period_seconds,
    })
}

fn query_subscription(deps: Deps, env: Env, subscriber: String) -> StdResult<SubscriptionResponse> {
    let subscriber = deps.api.addr_validate(&subscriber)?;
    let expires_at = SUBSCRIPTIONS
        .may_load(deps.storage, &subscriber)?
        .map(|subscription| subscription.expires_at)
        .unwrap_or_default();
    let now = env.block.time.seconds();
    let active = expires_at > now;

    Ok(SubscriptionResponse {
        subscriber,
        expires_at,
        active,
        remaining_seconds: expires_at.saturating_sub(now),
    })
}

fn query_is_active(deps: Deps, env: Env, subscriber: String) -> StdResult<IsActiveResponse> {
    Ok(IsActiveResponse {
        active: query_subscription(deps, env, subscriber)?.active,
    })
}

fn query_treasury(deps: Deps, env: Env) -> StdResult<TreasuryResponse> {
    let config = CONFIG.load(deps.storage)?;
    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    Ok(TreasuryResponse {
        denom: config.denom,
        balance: balance.amount,
    })
}

fn paid_amount(info: &MessageInfo, denom: &str) -> Result<Uint128, ContractError> {
    let mut paid = Uint128::zero();
    for coin in &info.funds {
        if coin.amount.is_zero() {
            continue;
        }
        if coin.denom != denom {
            return Err(ContractError::WrongDenom {});
        }
        paid += coin.amount;
    }
    Ok(paid)
}

fn validate_plan(price: Uint128, period_seconds: u64) -> Result<(), ContractError> {
    if price.is_zero() {
        return Err(ContractError::InvalidPrice {});
    }
    if period_seconds == 0 {
        return Err(ContractError::InvalidPeriod {});
    }
    Ok(())
}

fn assert_admin(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, coins, from_json, CosmosMsg};

    const DENOM: &str = "uatom";
    const PRICE: u128 = 1_000_000;
    const PERIOD: u64 = 2_592_000;

    fn instantiate_default(deps: DepsMut) {
        let msg = InstantiateMsg {
            denom: DENOM.to_string(),
            price: Uint128::new(PRICE),
            period_seconds: PERIOD,
            admin: None,
        };
        instantiate(deps, mock_env(), mock_info("admin", &[]), msg).unwrap();
    }

    #[test]
    fn instantiate_rejects_empty_plan() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            denom: DENOM.to_string(),
            price: Uint128::zero(),
            period_seconds: PERIOD,
            admin: None,
        };
        let err = instantiate(deps.as_mut(), mock_env(), mock_info("admin", &[]), msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidPrice {});

        let msg = InstantiateMsg {
            denom: DENOM.to_string(),
            price: Uint128::new(PRICE),
            period_seconds: 0,
            admin: None,
        };
        let err = instantiate(deps.as_mut(), mock_env(), mock_info("admin", &[]), msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidPeriod {});
    }

    #[test]
    fn subscribe_requires_expected_native_payment() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("subscriber", &coins(PRICE - 1, DENOM)),
            ExecuteMsg::Subscribe {},
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::MissingPayment {
                expected: Uint128::new(PRICE),
                denom: DENOM.to_string()
            }
        );

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("subscriber", &coins(PRICE, "uosmo")),
            ExecuteMsg::Subscribe {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::WrongDenom {});
    }

    #[test]
    fn subscribe_creates_and_extends_access() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut());

        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(100);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("subscriber", &coins(PRICE, DENOM)),
            ExecuteMsg::Subscribe {},
        )
        .unwrap();
        assert_eq!(res.attributes[0], attr("action", "subscribe"));

        let query_res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Subscription {
                subscriber: "subscriber".to_string(),
            },
        )
        .unwrap();
        let subscription: SubscriptionResponse = from_json(&query_res).unwrap();
        assert!(subscription.active);
        assert_eq!(subscription.expires_at, env.block.time.seconds() + PERIOD);
        assert_eq!(subscription.remaining_seconds, PERIOD);

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("subscriber", &coins(PRICE, DENOM)),
            ExecuteMsg::Subscribe {},
        )
        .unwrap();

        let query_res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Subscription {
                subscriber: "subscriber".to_string(),
            },
        )
        .unwrap();
        let subscription: SubscriptionResponse = from_json(&query_res).unwrap();
        assert_eq!(
            subscription.expires_at,
            env.block.time.seconds() + (PERIOD * 2)
        );
    }

    #[test]
    fn update_config_is_admin_only() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("subscriber", &[]),
            ExecuteMsg::UpdateConfig {
                price: Some(Uint128::new(2_000_000)),
                period_seconds: None,
                admin: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                price: Some(Uint128::new(2_000_000)),
                period_seconds: Some(100),
                admin: Some("new_admin".to_string()),
            },
        )
        .unwrap();

        let query_res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&query_res).unwrap();
        assert_eq!(config.price, Uint128::new(2_000_000));
        assert_eq!(config.period_seconds, 100);
        assert_eq!(config.admin, "new_admin");
    }

    #[test]
    fn admin_can_withdraw_collected_funds() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut());

        let mut env = mock_env();
        env.contract.address = deps.api.addr_make("contract");
        deps.querier
            .update_balance(env.contract.address.clone(), coins(PRICE, DENOM));

        let res = execute(
            deps.as_mut(),
            env,
            mock_info("admin", &[]),
            ExecuteMsg::Withdraw {
                recipient: Some("treasury".to_string()),
                amount: Some(Uint128::new(500_000)),
            },
        )
        .unwrap();

        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(500_000, DENOM));
            }
            _ => panic!("expected bank send"),
        }
    }
}
