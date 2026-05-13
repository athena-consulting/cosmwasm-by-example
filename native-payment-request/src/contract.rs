#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, InvoiceResponse, NextInvoiceIdResponse, QueryMsg,
};
use crate::state::{Config, Invoice, CONFIG, INVOICES, NEXT_ID};

const CONTRACT_NAME: &str = "crates.io:native-payment-request";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.denom.trim().is_empty() {
        return Err(ContractError::EmptyDenom {});
    }

    let merchant = match msg.merchant {
        Some(merchant) => deps.api.addr_validate(&merchant)?,
        None => info.sender.clone(),
    };
    let default_recipient = match msg.default_recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => merchant.clone(),
    };

    let config = Config {
        merchant,
        default_recipient,
        denom: msg.denom,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    NEXT_ID.save(deps.storage, &1)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("merchant", config.merchant)
        .add_attribute("default_recipient", config.default_recipient)
        .add_attribute("denom", config.denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateInvoice {
            id,
            payer,
            recipient,
            amount,
            description,
            due_at,
        } => execute_create_invoice(
            deps,
            env,
            info,
            CreateInvoiceArgs {
                id,
                payer,
                recipient,
                amount,
                description,
                due_at,
            },
        ),
        ExecuteMsg::PayInvoice { id } => execute_pay_invoice(deps, env, info, id),
        ExecuteMsg::CancelInvoice { id } => execute_cancel_invoice(deps, info, id),
        ExecuteMsg::UpdateConfig {
            merchant,
            default_recipient,
        } => execute_update_config(deps, info, merchant, default_recipient),
    }
}

struct CreateInvoiceArgs {
    id: Option<u64>,
    payer: Option<String>,
    recipient: Option<String>,
    amount: Uint128,
    description: String,
    due_at: Option<u64>,
}

fn execute_create_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    args: CreateInvoiceArgs,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_merchant(&info, &config)?;

    if args.amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    if let Some(due_at) = args.due_at {
        if due_at <= env.block.time.seconds() {
            return Err(ContractError::InvalidDueAt {});
        }
    }

    let id = match args.id {
        Some(id) => id,
        None => NEXT_ID.load(deps.storage)?,
    };

    if INVOICES.has(deps.storage, id) {
        return Err(ContractError::DuplicateInvoice {});
    }

    let payer = match args.payer {
        Some(payer) => Some(deps.api.addr_validate(&payer)?),
        None => None,
    };
    let recipient = match args.recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => config.default_recipient,
    };

    INVOICES.save(
        deps.storage,
        id,
        &Invoice {
            id,
            payer,
            recipient: recipient.clone(),
            amount: args.amount,
            description: args.description,
            due_at: args.due_at,
            paid: false,
            canceled: false,
            paid_at: None,
        },
    )?;

    NEXT_ID.update(deps.storage, |next_id| -> StdResult<_> {
        Ok(next_id.max(id.saturating_add(1)))
    })?;

    Ok(Response::new()
        .add_attribute("action", "create_invoice")
        .add_attribute("id", id.to_string())
        .add_attribute("recipient", recipient)
        .add_attribute("amount", args.amount)
        .add_attribute("denom", config.denom))
}

fn execute_pay_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut invoice = INVOICES.load(deps.storage, id)?;

    if invoice.canceled {
        return Err(ContractError::AlreadyCanceled {});
    }
    if invoice.paid {
        return Err(ContractError::AlreadyPaid {});
    }
    if let Some(due_at) = invoice.due_at {
        if due_at < env.block.time.seconds() {
            return Err(ContractError::InvoiceExpired {});
        }
    }
    if let Some(payer) = &invoice.payer {
        if payer != info.sender {
            return Err(ContractError::WrongPayer {});
        }
    }

    assert_exact_payment(&info, &config.denom, invoice.amount)?;

    invoice.paid = true;
    invoice.paid_at = Some(env.block.time.seconds());
    INVOICES.save(deps.storage, id, &invoice)?;

    let send = CosmosMsg::Bank(BankMsg::Send {
        to_address: invoice.recipient.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount: invoice.amount,
        }],
    });

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "pay_invoice")
        .add_attribute("id", id.to_string())
        .add_attribute("payer", info.sender)
        .add_attribute("recipient", invoice.recipient)
        .add_attribute("amount", invoice.amount)
        .add_attribute("denom", config.denom))
}

fn execute_cancel_invoice(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_merchant(&info, &config)?;

    INVOICES.update(deps.storage, id, |invoice| -> Result<_, ContractError> {
        let mut invoice =
            invoice.ok_or_else(|| cosmwasm_std::StdError::not_found(format!("invoice {id}")))?;
        if invoice.paid {
            return Err(ContractError::AlreadyPaid {});
        }
        if invoice.canceled {
            return Err(ContractError::AlreadyCanceled {});
        }
        invoice.canceled = true;
        Ok(invoice)
    })?;

    Ok(Response::new()
        .add_attribute("action", "cancel_invoice")
        .add_attribute("id", id.to_string()))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    merchant: Option<String>,
    default_recipient: Option<String>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        assert_merchant(&info, &config)?;

        if let Some(merchant) = merchant {
            config.merchant = deps.api.addr_validate(&merchant)?;
        }
        if let Some(default_recipient) = default_recipient {
            config.default_recipient = deps.api.addr_validate(&default_recipient)?;
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Invoice { id } => to_json_binary(&query_invoice(deps, id)?),
        QueryMsg::NextInvoiceId {} => to_json_binary(&NextInvoiceIdResponse {
            next_id: NEXT_ID.load(deps.storage)?,
        }),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        merchant: config.merchant,
        default_recipient: config.default_recipient,
        denom: config.denom,
    })
}

fn query_invoice(deps: Deps, id: u64) -> StdResult<InvoiceResponse> {
    let invoice = INVOICES.load(deps.storage, id)?;
    Ok(InvoiceResponse {
        id: invoice.id,
        payer: invoice.payer,
        recipient: invoice.recipient,
        amount: invoice.amount,
        description: invoice.description,
        due_at: invoice.due_at,
        paid: invoice.paid,
        canceled: invoice.canceled,
        paid_at: invoice.paid_at,
    })
}

fn assert_merchant(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.merchant {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn assert_exact_payment(
    info: &MessageInfo,
    denom: &str,
    expected: Uint128,
) -> Result<(), ContractError> {
    if info.funds.len() != 1 || info.funds[0].denom != denom || info.funds[0].amount != expected {
        let actual = info
            .funds
            .iter()
            .find(|coin| coin.denom == denom)
            .map(|coin| coin.amount)
            .unwrap_or_default();
        return Err(ContractError::InvalidPayment {
            expected,
            actual,
            denom: denom.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_json, Addr, BankMsg, CosmosMsg, Timestamp, Uint128};

    use super::*;

    fn setup() -> (
        cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        Env,
    ) {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("merchant", &[]),
            InstantiateMsg {
                denom: "uatom".to_string(),
                merchant: None,
                default_recipient: Some("treasury".to_string()),
            },
        )
        .unwrap();

        (deps, env)
    }

    fn create_invoice(
        deps: DepsMut,
        env: Env,
        payer: Option<String>,
        due_at: Option<u64>,
    ) -> Result<Response, ContractError> {
        execute(
            deps,
            env,
            mock_info("merchant", &[]),
            ExecuteMsg::CreateInvoice {
                id: None,
                payer,
                recipient: None,
                amount: Uint128::new(50),
                description: "consulting invoice".to_string(),
                due_at,
            },
        )
    }

    #[test]
    fn merchant_can_create_invoice() {
        let (mut deps, env) = setup();

        let res = create_invoice(
            deps.as_mut(),
            env.clone(),
            Some("payer".to_string()),
            Some(200),
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "create_invoice");

        let invoice: InvoiceResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Invoice { id: 1 }).unwrap()).unwrap();
        assert_eq!(invoice.payer, Some(Addr::unchecked("payer")));
        assert_eq!(invoice.recipient, Addr::unchecked("treasury"));
        assert_eq!(invoice.amount, Uint128::new(50));
        assert!(!invoice.paid);

        let next: NextInvoiceIdResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::NextInvoiceId {}).unwrap())
                .unwrap();
        assert_eq!(next.next_id, 2);
    }

    #[test]
    fn only_merchant_can_create_or_cancel() {
        let (mut deps, env) = setup();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("intruder", &[]),
            ExecuteMsg::CreateInvoice {
                id: None,
                payer: None,
                recipient: None,
                amount: Uint128::new(50),
                description: "not allowed".to_string(),
                due_at: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        create_invoice(deps.as_mut(), env.clone(), None, None).unwrap();
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("intruder", &[]),
            ExecuteMsg::CancelInvoice { id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn paying_invoice_forwards_funds_and_marks_paid() {
        let (mut deps, env) = setup();
        create_invoice(
            deps.as_mut(),
            env.clone(),
            Some("payer".to_string()),
            Some(200),
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("payer", &coins(50, "uatom")),
            ExecuteMsg::PayInvoice { id: 1 },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "treasury".to_string(),
                amount: vec![coin(50, "uatom")],
            })
        );

        let invoice: InvoiceResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Invoice { id: 1 }).unwrap()).unwrap();
        assert!(invoice.paid);
        assert_eq!(invoice.paid_at, Some(100));
    }

    #[test]
    fn payment_must_match_denom_amount_and_payer() {
        let (mut deps, env) = setup();
        create_invoice(deps.as_mut(), env.clone(), Some("payer".to_string()), None).unwrap();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("payer", &coins(49, "uatom")),
            ExecuteMsg::PayInvoice { id: 1 },
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidPayment {
                expected: Uint128::new(50),
                actual: Uint128::new(49),
                denom: "uatom".to_string(),
            }
        );

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("someone_else", &coins(50, "uatom")),
            ExecuteMsg::PayInvoice { id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::WrongPayer {});
    }

    #[test]
    fn expired_or_canceled_invoice_cannot_be_paid() {
        let (mut deps, mut env) = setup();
        create_invoice(deps.as_mut(), env.clone(), None, Some(150)).unwrap();

        env.block.time = Timestamp::from_seconds(151);
        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("payer", &coins(50, "uatom")),
            ExecuteMsg::PayInvoice { id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvoiceExpired {});

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        create_invoice(deps.as_mut(), env.clone(), None, None).unwrap();
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("merchant", &[]),
            ExecuteMsg::CancelInvoice { id: 2 },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("payer", &coins(50, "uatom")),
            ExecuteMsg::PayInvoice { id: 2 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AlreadyCanceled {});
    }
}
