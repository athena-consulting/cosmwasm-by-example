#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use std::collections::BTreeSet;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ContractBalanceResponse, ExecuteMsg, InstantiateMsg, Payment, QueryMsg,
};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-batch-distributor";
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

    let admin = match msg.admin {
        Some(admin) => deps.api.addr_validate(&admin)?,
        None => info.sender,
    };

    let config = Config {
        admin: admin.clone(),
        denom: denom.clone(),
    };
    CONFIG.save(deps.storage, &config)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin)
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
        ExecuteMsg::Distribute { payments } => execute_distribute(deps, env, info, payments),
        ExecuteMsg::UpdateAdmin { admin } => execute_update_admin(deps, info, admin),
    }
}

fn execute_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let payment = require_single_payment(&info.funds, &config.denom)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", payment.amount))
}

fn execute_distribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payments: Vec<Payment>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, &info)?;

    if payments.is_empty() {
        return Err(ContractError::EmptyPaymentList {});
    }

    let mut seen = BTreeSet::new();
    let mut total = Uint128::zero();
    let mut bank_messages = Vec::with_capacity(payments.len());

    for payment in payments {
        if payment.amount.is_zero() {
            return Err(ContractError::InvalidPaymentAmount {});
        }

        let recipient = deps.api.addr_validate(&payment.recipient)?;
        if !seen.insert(recipient.to_string()) {
            return Err(ContractError::DuplicateRecipient {});
        }

        total = total
            .checked_add(payment.amount)
            .map_err(StdError::overflow)?;
        bank_messages.push(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: payment.amount,
            }],
        });
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    if balance.amount < total {
        return Err(ContractError::InsufficientBalance {
            available: balance.amount,
            required: total,
        });
    }

    Ok(Response::new()
        .add_messages(bank_messages)
        .add_attribute("action", "distribute")
        .add_attribute("payment_count", seen.len().to_string())
        .add_attribute("denom", config.denom)
        .add_attribute("total", total))
}

fn execute_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    assert_admin(&config, &info)?;

    let new_admin = deps.api.addr_validate(&admin)?;
    if new_admin == config.admin {
        return Err(ContractError::SameAdmin {});
    }

    let previous_admin = config.admin;
    config.admin = new_admin.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("previous_admin", previous_admin)
        .add_attribute("new_admin", new_admin))
}

fn require_single_payment<'a>(
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
                return Err(ContractError::InvalidPaymentAmount {});
            }
            Ok(coin)
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

fn assert_admin(config: &Config, info: &MessageInfo) -> Result<(), ContractError> {
    if info.sender != config.admin {
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
        admin: config.admin.to_string(),
        denom: config.denom,
    })
}

fn query_balance(deps: Deps, env: Env) -> StdResult<ContractBalanceResponse> {
    let config = CONFIG.load(deps.storage)?;
    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    Ok(ContractBalanceResponse {
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
                admin: Some("admin".to_string()),
                denom: "uatom".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_rejects_empty_denom_and_defaults_admin() {
        let mut deps = mock_dependencies();

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                admin: None,
                denom: "   ".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EmptyDenom {}));

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                admin: None,
                denom: "uatom".to_string(),
            },
        )
        .unwrap();

        let response: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(response.admin, "creator");
        assert_eq!(response.denom, "uatom");
    }

    #[test]
    fn deposit_requires_single_positive_configured_denom() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MissingPayment {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[coin(100, "uosmo")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::WrongDenom { .. }));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[coin(100, "uatom"), coin(1, "uosmo")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MultipleDenoms {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[coin(0, "uatom")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidPaymentAmount {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sender", &[coin(250, "uatom")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "deposit");
        assert_eq!(res.messages.len(), 0);
    }

    #[test]
    fn only_admin_can_distribute() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(500, "uatom"));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::Distribute {
                payments: vec![Payment {
                    recipient: "alice".to_string(),
                    amount: Uint128::new(100),
                }],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn distribute_rejects_empty_zero_and_duplicate_recipients() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(500, "uatom"));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Distribute { payments: vec![] },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EmptyPaymentList {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Distribute {
                payments: vec![Payment {
                    recipient: "alice".to_string(),
                    amount: Uint128::zero(),
                }],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidPaymentAmount {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Distribute {
                payments: vec![
                    Payment {
                        recipient: "alice".to_string(),
                        amount: Uint128::new(100),
                    },
                    Payment {
                        recipient: "alice".to_string(),
                        amount: Uint128::new(100),
                    },
                ],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::DuplicateRecipient {}));
    }

    #[test]
    fn distribute_checks_balance_and_builds_bank_messages() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let payments = vec![
            Payment {
                recipient: "alice".to_string(),
                amount: Uint128::new(100),
            },
            Payment {
                recipient: "bob".to_string(),
                amount: Uint128::new(250),
            },
        ];

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Distribute {
                payments: payments.clone(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientBalance { .. }));

        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(500, "uatom"));
        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Distribute { payments },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 2);
        assert_eq!(res.attributes[1].value, "2");
        assert_eq!(res.attributes[3].value, "350");

        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(100, "uatom"));
            }
            _ => panic!("expected bank send"),
        }

        let response: ContractBalanceResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Balance {}).unwrap()).unwrap();
        assert_eq!(response.denom, "uatom");
        assert_eq!(response.amount, Uint128::new(500));
    }

    #[test]
    fn update_admin_changes_authority() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, coins(500, "uatom"));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateAdmin {
                admin: "new_admin".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Distribute {
                payments: vec![Payment {
                    recipient: "alice".to_string(),
                    amount: Uint128::new(10),
                }],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("new_admin", &[]),
            ExecuteMsg::Distribute {
                payments: vec![Payment {
                    recipient: "alice".to_string(),
                    amount: Uint128::new(10),
                }],
            },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);
    }
}
