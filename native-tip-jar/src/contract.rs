#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{BalanceResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-tip-jar";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let denom = msg.denom.trim().to_string();
    if denom.is_empty() {
        return Err(ContractError::EmptyDenom {});
    }

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            denom: denom.clone(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Tip {} => execute_tip(deps, info),
        ExecuteMsg::Withdraw { amount, recipient } => {
            execute_withdraw(deps, env, info, amount, recipient)
        }
        ExecuteMsg::UpdateOwner { owner } => execute_update_owner(deps, info, owner),
    }
}

fn execute_tip(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;

    Ok(Response::new()
        .add_attribute("action", "tip")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount))
}

fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info)?;

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    let amount = amount.unwrap_or(balance.amount);
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    if amount > balance.amount {
        return Err(ContractError::InsufficientBalance {
            available: balance.amount,
            requested: amount,
        });
    }

    let recipient = match recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => config.owner.clone(),
    };

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount,
            }],
        })
        .add_attribute("action", "withdraw")
        .add_attribute("recipient", recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", amount))
}

fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info)?;

    let new_owner = deps.api.addr_validate(&owner)?;
    if new_owner == config.owner {
        return Err(ContractError::SameOwner {});
    }

    let previous_owner = config.owner;
    config.owner = new_owner.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_owner")
        .add_attribute("previous_owner", previous_owner)
        .add_attribute("new_owner", new_owner))
}

fn require_single_coin<'a>(
    funds: &'a [Coin],
    expected_denom: &str,
) -> Result<&'a Coin, ContractError> {
    match funds {
        [] => Err(ContractError::MissingTip {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount.is_zero() {
                return Err(ContractError::InvalidAmount {});
            }
            Ok(coin)
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

fn assert_owner(config: &Config, info: &MessageInfo) -> Result<(), ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Balance {} => to_binary(&query_balance(deps, env)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
    })
}

fn query_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
    let config = CONFIG.load(deps.storage)?;
    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    Ok(BalanceResponse {
        denom: config.denom,
        amount: balance.amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coin, coins, from_binary, CosmosMsg};

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: Some("owner".to_string()),
                denom: "uatom".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_defaults_owner_and_rejects_empty_denom() {
        let mut deps = mock_dependencies();

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                denom: " ".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EmptyDenom {}));

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                denom: "uatom".to_string(),
            },
        )
        .unwrap();

        let response: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(response.owner, "creator");
        assert_eq!(response.denom, "uatom");
    }

    #[test]
    fn tip_requires_one_positive_configured_coin() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("tipper", &[]),
            ExecuteMsg::Tip {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MissingTip {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("tipper", &[coin(1, "uosmo")]),
            ExecuteMsg::Tip {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::WrongDenom { .. }));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("tipper", &[coin(1, "uatom"), coin(1, "uosmo")]),
            ExecuteMsg::Tip {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MultipleDenoms {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("tipper", &[coin(0, "uatom")]),
            ExecuteMsg::Tip {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidAmount {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("tipper", &[coin(25, "uatom")]),
            ExecuteMsg::Tip {},
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "tip");
        assert_eq!(res.messages.len(), 0);
    }

    #[test]
    fn only_owner_can_withdraw() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(100, "uatom"));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::Withdraw {
                amount: Some(Uint128::new(10)),
                recipient: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn withdraw_checks_amount_and_builds_bank_message() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(100, "uatom"));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                amount: Some(Uint128::zero()),
                recipient: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidAmount {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                amount: Some(Uint128::new(101)),
                recipient: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientBalance { .. }));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                amount: Some(Uint128::new(60)),
                recipient: Some("creator".to_string()),
            },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "creator");
                assert_eq!(amount, &coins(60, "uatom"));
            }
            _ => panic!("expected bank send"),
        }

        let response: BalanceResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Balance {}).unwrap()).unwrap();
        assert_eq!(response.amount, Uint128::new(100));
    }

    #[test]
    fn update_owner_moves_withdraw_authority() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(100, "uatom"));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateOwner {
                owner: "new_owner".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                amount: Some(Uint128::new(1)),
                recipient: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("new_owner", &[]),
            ExecuteMsg::Withdraw {
                amount: None,
                recipient: None,
            },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);
    }
}
