#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, CreditResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, CREDITS};

const CONTRACT_NAME: &str = "crates.io:native-service-credit";
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
    if msg.unit_price.is_zero() {
        return Err(ContractError::InvalidUnitPrice {});
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
            unit_price: msg.unit_price,
            balance: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("unit_price", msg.unit_price))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyCredits { account, credits } => {
            execute_buy_credits(deps, info, account, credits)
        }
        ExecuteMsg::TransferCredits { recipient, credits } => {
            execute_transfer_credits(deps, info, recipient, credits)
        }
        ExecuteMsg::DebitCredits {
            account,
            credits,
            reason,
        } => execute_debit_credits(deps, info, account, credits, reason),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
    }
}

fn execute_buy_credits(
    deps: DepsMut,
    info: MessageInfo,
    account: Option<String>,
    credits: u64,
) -> Result<Response, ContractError> {
    validate_credits(credits)?;
    let mut config = CONFIG.load(deps.storage)?;
    let required = config
        .unit_price
        .checked_mul(Uint128::from(credits as u128))
        .map_err(StdError::overflow)?;
    let coin = require_payment(&info.funds, &config.denom, required)?;
    let account = match account {
        Some(account) => deps.api.addr_validate(&account)?,
        None => info.sender.clone(),
    };

    CREDITS.update(
        deps.storage,
        &account,
        |current| -> Result<_, ContractError> {
            current
                .unwrap_or_default()
                .checked_add(credits)
                .ok_or_else(|| StdError::generic_err("credits overflow").into())
        },
    )?;
    config.balance = config
        .balance
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "buy_credits")
        .add_attribute("payer", info.sender)
        .add_attribute("account", account)
        .add_attribute("credits", credits.to_string())
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("balance", config.balance))
}

fn execute_transfer_credits(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    credits: u64,
) -> Result<Response, ContractError> {
    validate_credits(credits)?;
    let recipient = deps.api.addr_validate(&recipient)?;
    debit_account(deps.storage, &info.sender, credits)?;
    CREDITS.update(
        deps.storage,
        &recipient,
        |current| -> Result<_, ContractError> {
            current
                .unwrap_or_default()
                .checked_add(credits)
                .ok_or_else(|| StdError::generic_err("credits overflow").into())
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "transfer_credits")
        .add_attribute("from", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("credits", credits.to_string()))
}

fn execute_debit_credits(
    deps: DepsMut,
    info: MessageInfo,
    account: String,
    credits: u64,
    reason: Option<String>,
) -> Result<Response, ContractError> {
    validate_credits(credits)?;
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let account = deps.api.addr_validate(&account)?;
    debit_account(deps.storage, &account, credits)?;

    Ok(Response::new()
        .add_attribute("action", "debit_credits")
        .add_attribute("account", account)
        .add_attribute("credits", credits.to_string())
        .add_attribute("reason", reason.unwrap_or_default()))
}

fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidWithdrawAmount {});
    }
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if config.balance < amount {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: amount,
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
        .add_attribute("action", "withdraw")
        .add_attribute("recipient", recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", amount)
        .add_attribute("balance", config.balance))
}

fn validate_credits(credits: u64) -> Result<(), ContractError> {
    if credits == 0 {
        return Err(ContractError::InvalidCredits {});
    }
    Ok(())
}

fn debit_account(
    storage: &mut dyn cosmwasm_std::Storage,
    account: &cosmwasm_std::Addr,
    credits: u64,
) -> Result<(), ContractError> {
    CREDITS.update(storage, account, |current| -> Result<_, ContractError> {
        let current = current.unwrap_or_default();
        if current < credits {
            return Err(ContractError::InsufficientCredits {
                available: current,
                required: credits,
            });
        }
        Ok(current - credits)
    })?;
    Ok(())
}

fn require_payment<'a>(
    funds: &'a [Coin],
    expected_denom: &str,
    required: Uint128,
) -> Result<&'a Coin, ContractError> {
    match funds {
        [] => Err(ContractError::MissingPayment {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount < required {
                return Err(ContractError::Underpaid {
                    paid: coin.amount,
                    required,
                });
            }
            Ok(coin)
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Credit { account } => to_binary(&query_credit(deps, account)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        unit_price: config.unit_price,
        balance: config.balance,
    })
}

fn query_credit(deps: Deps, account: String) -> StdResult<CreditResponse> {
    let account_addr = deps.api.addr_validate(&account)?;
    Ok(CreditResponse {
        account,
        credits: CREDITS
            .may_load(deps.storage, &account_addr)?
            .unwrap_or_default(),
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
                unit_price: Uint128::new(5),
            },
        )
        .unwrap();
    }

    #[test]
    fn user_buys_and_transfers_credits() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(50, "uatom")),
            ExecuteMsg::BuyCredits {
                account: None,
                credits: 10,
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::TransferCredits {
                recipient: "bob".to_string(),
                credits: 4,
            },
        )
        .unwrap();

        let alice: CreditResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Credit {
                    account: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let bob: CreditResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Credit {
                    account: "bob".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(alice.credits, 6);
        assert_eq!(bob.credits, 4);
    }

    #[test]
    fn rejects_underpayment_and_wrong_denom() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let underpaid = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(49, "uatom")),
            ExecuteMsg::BuyCredits {
                account: None,
                credits: 10,
            },
        )
        .unwrap_err();
        assert!(matches!(underpaid, ContractError::Underpaid { .. }));

        let wrong_denom = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(50, "uosmo")),
            ExecuteMsg::BuyCredits {
                account: None,
                credits: 10,
            },
        )
        .unwrap_err();
        assert!(matches!(wrong_denom, ContractError::WrongDenom { .. }));
    }

    #[test]
    fn owner_debits_service_usage() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &coins(25, "uatom")),
            ExecuteMsg::BuyCredits {
                account: Some("alice".to_string()),
                credits: 5,
            },
        )
        .unwrap();

        let unauthorized = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::DebitCredits {
                account: "alice".to_string(),
                credits: 1,
                reason: None,
            },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::Unauthorized {}));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::DebitCredits {
                account: "alice".to_string(),
                credits: 3,
                reason: Some("support session".to_string()),
            },
        )
        .unwrap();

        let alice: CreditResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Credit {
                    account: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(alice.credits, 2);
    }

    #[test]
    fn owner_withdraws_collected_native_balance() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(50, "uatom")),
            ExecuteMsg::BuyCredits {
                account: None,
                credits: 10,
            },
        )
        .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(20),
            },
        )
        .unwrap();
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(20, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(30));
    }
}
