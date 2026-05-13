#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PoolResponse, QueryMsg, RewardResponse,
};
use crate::state::{Config, CONFIG, REWARDS, TOTAL_ALLOCATED};

const CONTRACT_NAME: &str = "crates.io:native-reward-pool";
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
    TOTAL_ALLOCATED.save(deps.storage, &Uint128::zero())?;
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
        ExecuteMsg::Deposit {} => execute_deposit(deps, info),
        ExecuteMsg::SetReward { recipient, amount } => {
            execute_set_reward(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Claim {} => execute_claim(deps, info),
        ExecuteMsg::UpdateOwner { owner } => execute_update_owner(deps, info, owner),
    }
}

fn execute_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount))
}

fn execute_set_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }

    let config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info)?;

    let recipient = deps.api.addr_validate(&recipient)?;
    let previous = REWARDS
        .may_load(deps.storage, &recipient)?
        .unwrap_or_default();
    let total_allocated = TOTAL_ALLOCATED.load(deps.storage)?;
    let updated_allocated = if amount >= previous {
        total_allocated
            .checked_add(amount - previous)
            .map_err(StdError::overflow)?
    } else {
        total_allocated
            .checked_sub(previous - amount)
            .map_err(StdError::overflow)?
    };

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    if updated_allocated > balance.amount {
        return Err(ContractError::InsufficientPool {
            balance: balance.amount,
            allocated: updated_allocated,
        });
    }

    REWARDS.save(deps.storage, &recipient, &amount)?;
    TOTAL_ALLOCATED.save(deps.storage, &updated_allocated)?;

    Ok(Response::new()
        .add_attribute("action", "set_reward")
        .add_attribute("recipient", recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", amount)
        .add_attribute("total_allocated", updated_allocated))
}

fn execute_claim(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let amount = REWARDS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();
    if amount.is_zero() {
        return Err(ContractError::NoReward {});
    }

    REWARDS.remove(deps.storage, &info.sender);
    let total_allocated = TOTAL_ALLOCATED
        .load(deps.storage)?
        .checked_sub(amount)
        .map_err(StdError::overflow)?;
    TOTAL_ALLOCATED.save(deps.storage, &total_allocated)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount,
            }],
        })
        .add_attribute("action", "claim")
        .add_attribute("recipient", info.sender)
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
        [] => Err(ContractError::MissingPayment {}),
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
        QueryMsg::Reward { recipient } => to_binary(&query_reward(deps, recipient)?),
        QueryMsg::Pool {} => to_binary(&query_pool(deps, env)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let total_allocated = TOTAL_ALLOCATED.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        total_allocated,
    })
}

fn query_reward(deps: Deps, recipient: String) -> StdResult<RewardResponse> {
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    let amount = REWARDS
        .may_load(deps.storage, &recipient_addr)?
        .unwrap_or_default();
    Ok(RewardResponse { recipient, amount })
}

fn query_pool(deps: Deps, env: Env) -> StdResult<PoolResponse> {
    let config = CONFIG.load(deps.storage)?;
    let total_allocated = TOTAL_ALLOCATED.load(deps.storage)?;
    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    Ok(PoolResponse {
        denom: config.denom,
        balance: balance.amount,
        total_allocated,
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
        assert_eq!(response.total_allocated, Uint128::zero());
    }

    #[test]
    fn deposit_requires_single_positive_configured_coin() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MissingPayment {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[coin(10, "uosmo")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::WrongDenom { .. }));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[coin(10, "uatom"), coin(1, "uosmo")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MultipleDenoms {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[coin(25, "uatom")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "deposit");
    }

    #[test]
    fn only_owner_can_set_rewards_and_pool_must_cover_allocations() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(100, "uatom"));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::SetReward {
                recipient: "alice".to_string(),
                amount: Uint128::new(10),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SetReward {
                recipient: "alice".to_string(),
                amount: Uint128::new(101),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientPool { .. }));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SetReward {
                recipient: "alice".to_string(),
                amount: Uint128::new(60),
            },
        )
        .unwrap();

        let reward: RewardResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Reward {
                    recipient: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(reward.amount, Uint128::new(60));
    }

    #[test]
    fn claim_clears_reward_and_sends_tokens() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(100, "uatom"));
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SetReward {
                recipient: "alice".to_string(),
                amount: Uint128::new(60),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NoReward {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap();
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(60, "uatom"));
            }
            _ => panic!("expected bank send"),
        }

        let pool: PoolResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Pool {}).unwrap()).unwrap();
        assert_eq!(pool.total_allocated, Uint128::zero());
    }

    #[test]
    fn update_owner_moves_allocation_authority() {
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
            ExecuteMsg::SetReward {
                recipient: "alice".to_string(),
                amount: Uint128::new(1),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("new_owner", &[]),
            ExecuteMsg::SetReward {
                recipient: "alice".to_string(),
                amount: Uint128::new(1),
            },
        )
        .unwrap();
    }
}
