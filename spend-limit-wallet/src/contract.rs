#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    AllowanceResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SpendableResponse,
};
use crate::state::{Allowance, Config, ALLOWANCES, CONFIG};

const CONTRACT_NAME: &str = "crates.io:spend-limit-wallet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.default_period == 0 {
        return Err(ContractError::InvalidPeriod {});
    }

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };

    let config = Config {
        owner,
        denom: msg.denom,
        default_period: msg.default_period,
    };

    CONFIG.save(deps.storage, &config)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", config.owner)
        .add_attribute("denom", config.denom)
        .add_attribute("default_period", config.default_period.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetAllowance {
            spender,
            limit,
            period,
        } => execute_set_allowance(deps, env, info, spender, limit, period),
        ExecuteMsg::RemoveAllowance { spender } => execute_remove_allowance(deps, info, spender),
        ExecuteMsg::Spend { recipient, amount } => {
            execute_spend(deps, env, info, recipient, amount)
        }
        ExecuteMsg::UpdateConfig {
            owner,
            default_period,
        } => execute_update_config(deps, info, owner, default_period),
    }
}

pub fn execute_set_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    limit: Uint128,
    period: Option<u64>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info.sender)?;

    if limit.is_zero() {
        return Err(ContractError::InvalidLimit {});
    }

    let period = period.unwrap_or(config.default_period);
    if period == 0 {
        return Err(ContractError::InvalidPeriod {});
    }

    let spender_addr = deps.api.addr_validate(&spender)?;
    let allowance = Allowance {
        limit,
        spent: Uint128::zero(),
        period_start: env.block.height,
        period,
    };

    ALLOWANCES.save(deps.storage, &spender_addr, &allowance)?;

    Ok(Response::new()
        .add_attribute("action", "set_allowance")
        .add_attribute("spender", spender_addr)
        .add_attribute("limit", limit)
        .add_attribute("period", period.to_string()))
}

pub fn execute_remove_allowance(
    deps: DepsMut,
    info: MessageInfo,
    spender: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info.sender)?;

    let spender_addr = deps.api.addr_validate(&spender)?;
    ALLOWANCES.remove(deps.storage, &spender_addr);

    Ok(Response::new()
        .add_attribute("action", "remove_allowance")
        .add_attribute("spender", spender_addr))
}

pub fn execute_spend(
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
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    let mut allowance = ALLOWANCES
        .may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::AllowanceNotFound {})?;

    reset_allowance_if_needed(&mut allowance, env.block.height);

    let remaining = remaining_amount(&allowance);
    if amount > remaining {
        return Err(ContractError::LimitExceeded {
            requested: amount,
            remaining,
        });
    }

    allowance.spent += amount;
    ALLOWANCES.save(deps.storage, &info.sender, &allowance)?;

    let send = BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "spend")
        .add_attribute("spender", info.sender)
        .add_attribute("recipient", recipient_addr)
        .add_attribute("amount", amount)
        .add_attribute("denom", config.denom))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    default_period: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info.sender)?;

    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(default_period) = default_period {
        if default_period == 0 {
            return Err(ContractError::InvalidPeriod {});
        }
        config.default_period = default_period;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute("default_period", config.default_period.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Allowance { spender } => to_json_binary(&query_allowance(deps, env, spender)?),
        QueryMsg::Spendable { spender } => to_json_binary(&query_spendable(deps, env, spender)?),
    }
}

pub fn query_config(deps: Deps) -> Result<ConfigResponse, StdError> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        default_period: config.default_period,
    })
}

pub fn query_allowance(
    deps: Deps,
    env: Env,
    spender: String,
) -> Result<AllowanceResponse, StdError> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    let mut allowance = ALLOWANCES.load(deps.storage, &spender_addr)?;
    reset_allowance_if_needed(&mut allowance, env.block.height);

    Ok(AllowanceResponse {
        spender: spender_addr.to_string(),
        limit: allowance.limit,
        spent: allowance.spent,
        remaining: remaining_amount(&allowance),
        period_start: allowance.period_start,
        period: allowance.period,
    })
}

pub fn query_spendable(
    deps: Deps,
    env: Env,
    spender: String,
) -> Result<SpendableResponse, StdError> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    let mut allowance = ALLOWANCES.load(deps.storage, &spender_addr)?;
    reset_allowance_if_needed(&mut allowance, env.block.height);

    Ok(SpendableResponse {
        spender: spender_addr.to_string(),
        amount: remaining_amount(&allowance),
        period_start: allowance.period_start,
        period: allowance.period,
    })
}

fn assert_owner(config: &Config, sender: &Addr) -> Result<(), ContractError> {
    if sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

fn reset_allowance_if_needed(allowance: &mut Allowance, block_height: u64) {
    if block_height >= allowance.period_start.saturating_add(allowance.period) {
        allowance.spent = Uint128::zero();
        allowance.period_start = block_height;
    }
}

fn remaining_amount(allowance: &Allowance) -> Uint128 {
    allowance
        .limit
        .checked_sub(allowance.spent)
        .unwrap_or_else(|_| Uint128::zero())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_json, CosmosMsg, SubMsg};

    fn instantiate_wallet(deps: DepsMut) {
        let msg = InstantiateMsg {
            owner: None,
            denom: "uatom".to_string(),
            default_period: 10,
        };
        let info = mock_info("owner", &[]);

        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn instantiate_saves_config() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        let response: ConfigResponse =
            from_json(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();

        assert_eq!(response.owner, "owner");
        assert_eq!(response.denom, "uatom");
        assert_eq!(response.default_period, 10);
    }

    #[test]
    fn owner_sets_allowance() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        let msg = ExecuteMsg::SetAllowance {
            spender: "spender".to_string(),
            limit: Uint128::new(100),
            period: Some(5),
        };
        let res = execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();

        assert_eq!(res.attributes[0].value, "set_allowance");

        let response: AllowanceResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Allowance {
                    spender: "spender".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(response.limit, Uint128::new(100));
        assert_eq!(response.remaining, Uint128::new(100));
        assert_eq!(response.period, 5);
    }

    #[test]
    fn non_owner_cannot_set_allowance() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        let msg = ExecuteMsg::SetAllowance {
            spender: "spender".to_string(),
            limit: Uint128::new(100),
            period: None,
        };
        let err = execute(deps.as_mut(), mock_env(), mock_info("other", &[]), msg).unwrap_err();

        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn spender_can_send_within_limit() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SetAllowance {
                spender: "spender".to_string(),
                limit: Uint128::new(100),
                period: None,
            },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("spender", &[]),
            ExecuteMsg::Spend {
                recipient: "recipient".to_string(),
                amount: Uint128::new(40),
            },
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "recipient".to_string(),
                amount: vec![Coin {
                    denom: "uatom".to_string(),
                    amount: Uint128::new(40),
                }],
            }))]
        );

        let response: SpendableResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Spendable {
                    spender: "spender".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(response.amount, Uint128::new(60));
    }

    #[test]
    fn spend_fails_over_limit() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SetAllowance {
                spender: "spender".to_string(),
                limit: Uint128::new(30),
                period: None,
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("spender", &[]),
            ExecuteMsg::Spend {
                recipient: "recipient".to_string(),
                amount: Uint128::new(31),
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::LimitExceeded {
                requested: Uint128::new(31),
                remaining: Uint128::new(30),
            }
        );
    }

    #[test]
    fn allowance_resets_after_period() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SetAllowance {
                spender: "spender".to_string(),
                limit: Uint128::new(50),
                period: Some(7),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("spender", &[]),
            ExecuteMsg::Spend {
                recipient: "recipient".to_string(),
                amount: Uint128::new(45),
            },
        )
        .unwrap();

        let mut later = mock_env();
        later.block.height += 7;

        let response: SpendableResponse = from_json(
            &query(
                deps.as_ref(),
                later,
                QueryMsg::Spendable {
                    spender: "spender".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(response.amount, Uint128::new(50));
    }

    #[test]
    fn removed_allowance_cannot_spend() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SetAllowance {
                spender: "spender".to_string(),
                limit: Uint128::new(50),
                period: None,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::RemoveAllowance {
                spender: "spender".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("spender", &[]),
            ExecuteMsg::Spend {
                recipient: "recipient".to_string(),
                amount: Uint128::new(1),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::AllowanceNotFound {});
    }

    #[test]
    fn owner_updates_config() {
        let mut deps = mock_dependencies();
        instantiate_wallet(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                owner: Some("new_owner".to_string()),
                default_period: Some(25),
            },
        )
        .unwrap();

        let response: ConfigResponse =
            from_json(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();

        assert_eq!(response.owner, "new_owner");
        assert_eq!(response.default_period, 25);
    }
}
