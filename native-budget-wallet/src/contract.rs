#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{BudgetResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Budget, Config, BUDGETS, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-budget-wallet";
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
            next_budget_id: 1,
            balance: Uint128::zero(),
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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Fund {} => execute_fund(deps, info),
        ExecuteMsg::CreateBudget {
            manager,
            label,
            cap,
        } => execute_create_budget(deps, info, manager, label, cap),
        ExecuteMsg::ClaimBudget {
            budget_id,
            recipient,
            amount,
            memo,
        } => execute_claim_budget(deps, info, budget_id, recipient, amount, memo),
        ExecuteMsg::CloseBudget { budget_id } => execute_close_budget(deps, info, budget_id),
        ExecuteMsg::WithdrawUnreserved { recipient, amount } => {
            execute_withdraw_unreserved(deps, info, recipient, amount)
        }
    }
}

fn execute_fund(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let coin = require_payment(&info.funds, &config.denom)?;
    config.balance = config
        .balance
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "fund")
        .add_attribute("sender", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("balance", config.balance))
}

fn execute_create_budget(
    deps: DepsMut,
    info: MessageInfo,
    manager: String,
    label: String,
    cap: Uint128,
) -> Result<Response, ContractError> {
    if cap.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    let label = label.trim().to_string();
    if label.is_empty() {
        return Err(ContractError::EmptyLabel {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let manager = deps.api.addr_validate(&manager)?;
    let budget_id = config.next_budget_id;

    BUDGETS.save(
        deps.storage,
        budget_id,
        &Budget {
            id: budget_id,
            manager: manager.clone(),
            label: label.clone(),
            cap,
            spent: Uint128::zero(),
            active: true,
        },
    )?;
    config.next_budget_id = config
        .next_budget_id
        .checked_add(1)
        .ok_or_else(|| StdError::generic_err("budget id overflow"))?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "create_budget")
        .add_attribute("budget_id", budget_id.to_string())
        .add_attribute("manager", manager)
        .add_attribute("label", label)
        .add_attribute("cap", cap))
}

fn execute_claim_budget(
    deps: DepsMut,
    info: MessageInfo,
    budget_id: u64,
    recipient: Option<String>,
    amount: Uint128,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    let mut budget = BUDGETS.load(deps.storage, budget_id)?;
    if !budget.active {
        return Err(ContractError::BudgetClosed {});
    }
    if info.sender != budget.manager {
        return Err(ContractError::NotManager {});
    }

    let remaining = budget_remaining(&budget)?;
    if remaining < amount {
        return Err(ContractError::BudgetExceeded {
            remaining,
            requested: amount,
        });
    }
    let mut config = CONFIG.load(deps.storage)?;
    if config.balance < amount {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: amount,
        });
    }

    let recipient = match recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => budget.manager.clone(),
    };
    budget.spent = budget
        .spent
        .checked_add(amount)
        .map_err(StdError::overflow)?;
    config.balance = config
        .balance
        .checked_sub(amount)
        .map_err(StdError::overflow)?;
    BUDGETS.save(deps.storage, budget_id, &budget)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount,
            }],
        })
        .add_attribute("action", "claim_budget")
        .add_attribute("budget_id", budget_id.to_string())
        .add_attribute("manager", budget.manager)
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount)
        .add_attribute("memo", memo.unwrap_or_default())
        .add_attribute("balance", config.balance))
}

fn execute_close_budget(
    deps: DepsMut,
    info: MessageInfo,
    budget_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let mut budget = BUDGETS.load(deps.storage, budget_id)?;
    budget.active = false;
    BUDGETS.save(deps.storage, budget_id, &budget)?;

    Ok(Response::new()
        .add_attribute("action", "close_budget")
        .add_attribute("budget_id", budget_id.to_string()))
}

fn execute_withdraw_unreserved(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let reserved = reserved_balance(deps.as_ref())?;
    let unreserved = config.balance.checked_sub(reserved).unwrap_or_default();
    if unreserved < amount {
        return Err(ContractError::InsufficientUnreserved {
            available: unreserved,
            requested: amount,
        });
    }
    let recipient = deps.api.addr_validate(&recipient)?;
    config.balance = config
        .balance
        .checked_sub(amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount,
            }],
        })
        .add_attribute("action", "withdraw_unreserved")
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount)
        .add_attribute("balance", config.balance))
}

fn require_payment<'a>(funds: &'a [Coin], expected_denom: &str) -> Result<&'a Coin, ContractError> {
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

fn budget_remaining(budget: &Budget) -> StdResult<Uint128> {
    budget
        .cap
        .checked_sub(budget.spent)
        .map_err(StdError::overflow)
}

fn reserved_balance(deps: Deps) -> StdResult<Uint128> {
    BUDGETS
        .range(deps.storage, None, None, Order::Ascending)
        .try_fold(Uint128::zero(), |sum, item| {
            let (_, budget) = item?;
            if budget.active {
                sum.checked_add(budget_remaining(&budget)?)
                    .map_err(StdError::overflow)
            } else {
                Ok(sum)
            }
        })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Budget { budget_id } => to_binary(&query_budget(deps, budget_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let reserved = reserved_balance(deps)?;
    let unreserved = config.balance.checked_sub(reserved).unwrap_or_default();
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        next_budget_id: config.next_budget_id,
        balance: config.balance,
        reserved,
        unreserved,
    })
}

fn query_budget(deps: Deps, budget_id: u64) -> StdResult<BudgetResponse> {
    let budget = BUDGETS.load(deps.storage, budget_id)?;
    let remaining = budget_remaining(&budget)?;
    Ok(BudgetResponse {
        id: budget.id,
        manager: budget.manager.to_string(),
        label: budget.label,
        cap: budget.cap,
        spent: budget.spent,
        remaining,
        active: budget.active,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, CosmosMsg};

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

    fn fund_and_create_budget(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("funder", &coins(100, "uatom")),
            ExecuteMsg::Fund {},
        )
        .unwrap();
    }

    fn create_budget(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CreateBudget {
                manager: "manager".to_string(),
                label: "events".to_string(),
                cap: Uint128::new(60),
            },
        )
        .unwrap();
    }

    #[test]
    fn manager_claims_from_budget() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        fund_and_create_budget(deps.as_mut());
        create_budget(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("manager", &[]),
            ExecuteMsg::ClaimBudget {
                budget_id: 1,
                recipient: Some("vendor".to_string()),
                amount: Uint128::new(25),
                memo: Some("venue deposit".to_string()),
            },
        )
        .unwrap();
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "vendor");
                assert_eq!(amount, &coins(25, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let budget: BudgetResponse = from_binary(
            &query(deps.as_ref(), mock_env(), QueryMsg::Budget { budget_id: 1 }).unwrap(),
        )
        .unwrap();
        assert_eq!(budget.spent, Uint128::new(25));
        assert_eq!(budget.remaining, Uint128::new(35));
    }

    #[test]
    fn rejects_unauthorized_actions() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let create = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::CreateBudget {
                manager: "manager".to_string(),
                label: "events".to_string(),
                cap: Uint128::new(60),
            },
        )
        .unwrap_err();
        assert!(matches!(create, ContractError::Unauthorized {}));

        fund_and_create_budget(deps.as_mut());
        create_budget(deps.as_mut());
        let claim = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::ClaimBudget {
                budget_id: 1,
                recipient: None,
                amount: Uint128::new(10),
                memo: None,
            },
        )
        .unwrap_err();
        assert!(matches!(claim, ContractError::NotManager {}));
    }

    #[test]
    fn prevents_overclaim_and_wrong_denom_funding() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let wrong_denom = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("funder", &coins(100, "uosmo")),
            ExecuteMsg::Fund {},
        )
        .unwrap_err();
        assert!(matches!(wrong_denom, ContractError::WrongDenom { .. }));

        fund_and_create_budget(deps.as_mut());
        create_budget(deps.as_mut());
        let overclaim = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("manager", &[]),
            ExecuteMsg::ClaimBudget {
                budget_id: 1,
                recipient: None,
                amount: Uint128::new(61),
                memo: None,
            },
        )
        .unwrap_err();
        assert!(matches!(overclaim, ContractError::BudgetExceeded { .. }));
    }

    #[test]
    fn owner_withdraws_only_unreserved_balance() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        fund_and_create_budget(deps.as_mut());
        create_budget(deps.as_mut());

        let reserved_withdrawal = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::WithdrawUnreserved {
                recipient: "treasury".to_string(),
                amount: Uint128::new(50),
            },
        )
        .unwrap_err();
        assert!(matches!(
            reserved_withdrawal,
            ContractError::InsufficientUnreserved { .. }
        ));

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::WithdrawUnreserved {
                recipient: "treasury".to_string(),
                amount: Uint128::new(40),
            },
        )
        .unwrap();
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(40, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(60));
        assert_eq!(config.reserved, Uint128::new(60));
        assert_eq!(config.unreserved, Uint128::zero());
    }
}
