#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, InvoiceResponse, QueryMsg};
use crate::state::{Config, Invoice, CONFIG, INVOICES};

const CONTRACT_NAME: &str = "crates.io:native-invoice-book";
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
            balance: Uint128::zero(),
            next_invoice_id: 1,
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
        ExecuteMsg::CreateInvoice {
            payer,
            amount,
            memo,
        } => execute_create_invoice(deps, info, payer, amount, memo),
        ExecuteMsg::PayInvoice { invoice_id } => execute_pay_invoice(deps, info, invoice_id),
        ExecuteMsg::CancelInvoice { invoice_id } => execute_cancel_invoice(deps, info, invoice_id),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
    }
}

fn execute_create_invoice(
    deps: DepsMut,
    info: MessageInfo,
    payer: Option<String>,
    amount: Uint128,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidInvoiceAmount {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let payer = match payer {
        Some(payer) => Some(deps.api.addr_validate(&payer)?),
        None => None,
    };
    let invoice_id = config.next_invoice_id;
    config.next_invoice_id += 1;

    INVOICES.save(
        deps.storage,
        invoice_id,
        &Invoice {
            payer: payer.clone(),
            amount,
            memo,
            paid: false,
            paid_by: None,
        },
    )?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "create_invoice")
        .add_attribute("invoice_id", invoice_id.to_string())
        .add_attribute(
            "payer",
            payer.map(|addr| addr.to_string()).unwrap_or_default(),
        )
        .add_attribute("amount", amount))
}

fn execute_pay_invoice(
    deps: DepsMut,
    info: MessageInfo,
    invoice_id: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut invoice = INVOICES
        .may_load(deps.storage, invoice_id)?
        .ok_or(ContractError::InvoiceNotFound {})?;
    if invoice.paid {
        return Err(ContractError::InvoiceAlreadyPaid {});
    }
    if let Some(payer) = &invoice.payer {
        if info.sender != payer {
            return Err(ContractError::WrongPayer {});
        }
    }

    let coin = require_invoice_payment(&info.funds, &config.denom, invoice.amount)?;
    config.balance = config
        .balance
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    invoice.paid = true;
    invoice.paid_by = Some(info.sender.clone());
    CONFIG.save(deps.storage, &config)?;
    INVOICES.save(deps.storage, invoice_id, &invoice)?;

    Ok(Response::new()
        .add_attribute("action", "pay_invoice")
        .add_attribute("invoice_id", invoice_id.to_string())
        .add_attribute("payer", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("balance", config.balance))
}

fn execute_cancel_invoice(
    deps: DepsMut,
    info: MessageInfo,
    invoice_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let invoice = INVOICES
        .may_load(deps.storage, invoice_id)?
        .ok_or(ContractError::InvoiceNotFound {})?;
    if invoice.paid {
        return Err(ContractError::InvoiceAlreadyPaid {});
    }

    INVOICES.remove(deps.storage, invoice_id);

    Ok(Response::new()
        .add_attribute("action", "cancel_invoice")
        .add_attribute("invoice_id", invoice_id.to_string()))
}

fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if amount.is_zero() {
        return Err(ContractError::InvalidWithdrawAmount {});
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

fn require_invoice_payment<'a>(
    funds: &'a [Coin],
    expected_denom: &str,
    required_amount: Uint128,
) -> Result<&'a Coin, ContractError> {
    match funds {
        [] => Err(ContractError::MissingPayment {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount < required_amount {
                return Err(ContractError::Underpaid {
                    paid: coin.amount,
                    required: required_amount,
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
        QueryMsg::Invoice { invoice_id } => to_binary(&query_invoice(deps, invoice_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        balance: config.balance,
        next_invoice_id: config.next_invoice_id,
    })
}

fn query_invoice(deps: Deps, invoice_id: u64) -> StdResult<InvoiceResponse> {
    let invoice = INVOICES.load(deps.storage, invoice_id)?;
    Ok(InvoiceResponse {
        invoice_id,
        payer: invoice.payer.map(|addr| addr.to_string()),
        amount: invoice.amount,
        memo: invoice.memo,
        paid: invoice.paid,
        paid_by: invoice.paid_by.map(|addr| addr.to_string()),
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

    fn create_invoice(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CreateInvoice {
                payer: Some("alice".to_string()),
                amount: Uint128::new(120),
                memo: Some("design review".to_string()),
            },
        )
        .unwrap();
    }

    #[test]
    fn owner_creates_and_payer_settles_invoice() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_invoice(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(150, "uatom")),
            ExecuteMsg::PayInvoice { invoice_id: 1 },
        )
        .unwrap();
        assert_eq!(response.messages.len(), 0);

        let invoice: InvoiceResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Invoice { invoice_id: 1 },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(invoice.paid);
        assert_eq!(invoice.paid_by, Some("alice".to_string()));

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(150));
    }

    #[test]
    fn rejects_wrong_payer_and_underpayment() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_invoice(deps.as_mut());

        let wrong_payer = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &coins(120, "uatom")),
            ExecuteMsg::PayInvoice { invoice_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(wrong_payer, ContractError::WrongPayer {}));

        let underpaid = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(119, "uatom")),
            ExecuteMsg::PayInvoice { invoice_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(underpaid, ContractError::Underpaid { .. }));
    }

    #[test]
    fn owner_withdraws_collected_invoice_balance() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_invoice(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(120, "uatom")),
            ExecuteMsg::PayInvoice { invoice_id: 1 },
        )
        .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(50),
            },
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(50, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(70));
    }

    #[test]
    fn owner_cancels_only_unpaid_invoices() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_invoice(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CancelInvoice { invoice_id: 1 },
        )
        .unwrap();

        let err = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Invoice { invoice_id: 1 },
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }
}
