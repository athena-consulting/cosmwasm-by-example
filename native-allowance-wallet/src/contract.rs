#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{AllowanceResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-allowance-wallet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.denom.trim().is_empty() {
        return Err(ContractError::EmptyDenom {});
    }
    assert_valid_period(msg.period_seconds)?;
    assert_positive("limit", msg.period_limit)?;

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender.clone(),
    };
    let spender = deps.api.addr_validate(&msg.spender)?;

    let config = Config {
        owner,
        spender,
        denom: msg.denom,
        period_seconds: msg.period_seconds,
        period_limit: msg.period_limit,
        current_period_start: env.block.time.seconds(),
        spent_this_period: Uint128::zero(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", config.owner)
        .add_attribute("spender", config.spender)
        .add_attribute("denom", config.denom)
        .add_attribute("period_seconds", config.period_seconds.to_string())
        .add_attribute("period_limit", config.period_limit))
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
        ExecuteMsg::Withdraw { amount, recipient } => {
            execute_withdraw(deps, env, info, amount, recipient)
        }
        ExecuteMsg::UpdateConfig {
            owner,
            spender,
            period_seconds,
            period_limit,
            reset_period,
        } => execute_update_config(
            deps,
            env,
            info,
            owner,
            spender,
            period_seconds,
            period_limit,
            reset_period,
        ),
    }
}

fn execute_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let amount = assert_exact_denom_payment(&info, &config.denom)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount)
        .add_attribute("denom", config.denom))
}

fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    assert_positive("amount", amount)?;

    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.spender {
        return Err(ContractError::Unauthorized {});
    }

    refresh_period(&mut config, env.block.time.seconds());
    let remaining = remaining_allowance(&config);
    if amount > remaining {
        return Err(ContractError::AllowanceExceeded {
            remaining,
            requested: amount,
        });
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?
        .amount;
    if amount > balance {
        return Err(ContractError::InsufficientBalance {
            available: balance,
            requested: amount,
        });
    }

    config.spent_this_period += amount;
    CONFIG.save(deps.storage, &config)?;

    let recipient = match recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => info.sender.clone(),
    };
    let send = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount,
        }],
    });

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "withdraw")
        .add_attribute("spender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount)
        .add_attribute("denom", config.denom)
        .add_attribute("remaining_this_period", remaining - amount))
}

#[allow(clippy::too_many_arguments)]
fn execute_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    spender: Option<String>,
    period_seconds: Option<u64>,
    period_limit: Option<Uint128>,
    reset_period: bool,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }

        let mut should_reset_period = reset_period;
        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(spender) = spender {
            config.spender = deps.api.addr_validate(&spender)?;
        }
        if let Some(period_seconds) = period_seconds {
            assert_valid_period(period_seconds)?;
            config.period_seconds = period_seconds;
            should_reset_period = true;
        }
        if let Some(period_limit) = period_limit {
            assert_positive("limit", period_limit)?;
            config.period_limit = period_limit;
        }
        if should_reset_period {
            config.current_period_start = env.block.time.seconds();
            config.spent_this_period = Uint128::zero();
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Allowance {} => to_json_binary(&query_allowance(deps, env)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        spender: config.spender,
        denom: config.denom,
        period_seconds: config.period_seconds,
        period_limit: config.period_limit,
    })
}

fn query_allowance(deps: Deps, env: Env) -> StdResult<AllowanceResponse> {
    let mut config = CONFIG.load(deps.storage)?;
    refresh_period(&mut config, env.block.time.seconds());
    let contract_balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?
        .amount;
    let remaining_this_period = remaining_allowance(&config);

    Ok(AllowanceResponse {
        owner: config.owner,
        spender: config.spender,
        denom: config.denom.clone(),
        current_period_start: config.current_period_start,
        current_period_end: config
            .current_period_start
            .saturating_add(config.period_seconds),
        period_limit: config.period_limit,
        spent_this_period: config.spent_this_period,
        remaining_this_period,
        contract_balance,
    })
}

fn refresh_period(config: &mut Config, now: u64) {
    let period_end = config
        .current_period_start
        .saturating_add(config.period_seconds);
    if now < period_end {
        return;
    }

    let elapsed = now.saturating_sub(config.current_period_start);
    let periods_elapsed = elapsed / config.period_seconds;
    config.current_period_start += periods_elapsed * config.period_seconds;
    config.spent_this_period = Uint128::zero();
}

fn remaining_allowance(config: &Config) -> Uint128 {
    config
        .period_limit
        .checked_sub(config.spent_this_period)
        .unwrap_or_default()
}

fn assert_exact_denom_payment(info: &MessageInfo, denom: &str) -> Result<Uint128, ContractError> {
    if info.funds.len() != 1 || info.funds[0].denom != denom || info.funds[0].amount.is_zero() {
        let actual = info
            .funds
            .iter()
            .find(|coin| coin.denom == denom)
            .map(|coin| coin.amount)
            .unwrap_or_default();
        return Err(ContractError::InvalidDeposit {
            denom: denom.to_string(),
            actual,
        });
    }

    Ok(info.funds[0].amount)
}

fn assert_valid_period(period_seconds: u64) -> Result<(), ContractError> {
    if period_seconds == 0 {
        return Err(ContractError::InvalidPeriod {});
    }
    Ok(())
}

fn assert_positive(kind: &str, amount: Uint128) -> Result<(), ContractError> {
    if amount.is_zero() {
        return match kind {
            "limit" => Err(ContractError::InvalidLimit {}),
            _ => Err(ContractError::InvalidAmount {}),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coin, coins, from_json, Addr, BankMsg, CosmosMsg, Timestamp};

    use super::*;

    fn setup_with_balance(
        amount: u128,
    ) -> (
        cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        Env,
    ) {
        let mut deps = mock_dependencies_with_balance(&coins(amount, "uatom"));
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            InstantiateMsg {
                denom: "uatom".to_string(),
                owner: None,
                spender: "spender".to_string(),
                period_seconds: 60,
                period_limit: Uint128::new(40),
            },
        )
        .unwrap();

        (deps, env)
    }

    #[test]
    fn instantiate_stores_config() {
        let (deps, env) = setup_with_balance(100);

        let config: ConfigResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.owner, Addr::unchecked("owner"));
        assert_eq!(config.spender, Addr::unchecked("spender"));
        assert_eq!(config.denom, "uatom");
        assert_eq!(config.period_seconds, 60);
        assert_eq!(config.period_limit, Uint128::new(40));
    }

    #[test]
    fn deposit_requires_configured_native_denom() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        instantiate(
            deps.as_mut(),
            env,
            mock_info("owner", &[]),
            InstantiateMsg {
                denom: "uatom".to_string(),
                owner: None,
                spender: "spender".to_string(),
                period_seconds: 60,
                period_limit: Uint128::new(40),
            },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("supporter", &coins(25, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "deposit");
        assert_eq!(res.attributes[2].value, "25");

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("supporter", &coins(25, "uosmo")),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidDeposit {
                denom: "uatom".to_string(),
                actual: Uint128::zero(),
            }
        );
    }

    #[test]
    fn spender_withdraws_within_current_period_limit() {
        let (mut deps, env) = setup_with_balance(100);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("spender", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(30),
                recipient: Some("vendor".to_string()),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "vendor".to_string(),
                amount: vec![coin(30, "uatom")],
            })
        );

        let allowance: AllowanceResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Allowance {}).unwrap()).unwrap();
        assert_eq!(allowance.spent_this_period, Uint128::new(30));
        assert_eq!(allowance.remaining_this_period, Uint128::new(10));
        assert_eq!(allowance.contract_balance, Uint128::new(100));
    }

    #[test]
    fn withdraw_enforces_spender_limit_and_contract_balance() {
        let (mut deps, env) = setup_with_balance(35);

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("intruder", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(1),
                recipient: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("spender", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(41),
                recipient: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::AllowanceExceeded {
                remaining: Uint128::new(40),
                requested: Uint128::new(41),
            }
        );

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("spender", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(36),
                recipient: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::InsufficientBalance {
                available: Uint128::new(35),
                requested: Uint128::new(36),
            }
        );
    }

    #[test]
    fn allowance_resets_when_period_rolls_forward() {
        let (mut deps, mut env) = setup_with_balance(100);

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("spender", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(40),
                recipient: None,
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("spender", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(1),
                recipient: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::AllowanceExceeded {
                remaining: Uint128::zero(),
                requested: Uint128::new(1),
            }
        );

        env.block.time = Timestamp::from_seconds(160);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("spender", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(15),
                recipient: None,
            },
        )
        .unwrap();
        assert_eq!(res.attributes[5].value, "25");

        let allowance: AllowanceResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Allowance {}).unwrap()).unwrap();
        assert_eq!(allowance.current_period_start, 160);
        assert_eq!(allowance.spent_this_period, Uint128::new(15));
        assert_eq!(allowance.remaining_this_period, Uint128::new(25));
    }

    #[test]
    fn owner_can_update_config_and_reset_period() {
        let (mut deps, env) = setup_with_balance(100);

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("spender", &[]),
            ExecuteMsg::Withdraw {
                amount: Uint128::new(20),
                recipient: None,
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("spender", &[]),
            ExecuteMsg::UpdateConfig {
                owner: None,
                spender: Some("new_spender".to_string()),
                period_seconds: None,
                period_limit: None,
                reset_period: true,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                owner: None,
                spender: Some("new_spender".to_string()),
                period_seconds: Some(120),
                period_limit: Some(Uint128::new(80)),
                reset_period: true,
            },
        )
        .unwrap();

        let allowance: AllowanceResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Allowance {}).unwrap()).unwrap();
        assert_eq!(allowance.spender, Addr::unchecked("new_spender"));
        assert_eq!(allowance.period_limit, Uint128::new(80));
        assert_eq!(allowance.current_period_end, 220);
        assert_eq!(allowance.spent_this_period, Uint128::zero());
    }
}
