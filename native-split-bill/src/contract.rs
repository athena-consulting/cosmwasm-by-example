#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    BillResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ShareStatusResponse,
};
use crate::state::{Bill, BillStatus, Config, BILLS, CONFIG, PAID};

const CONTRACT_NAME: &str = "crates.io:native-split-bill";
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
            next_bill_id: 1,
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
        ExecuteMsg::CreateBill {
            recipient,
            participants,
            share_amount,
            memo,
        } => execute_create_bill(deps, info, recipient, participants, share_amount, memo),
        ExecuteMsg::PayShare { bill_id } => execute_pay_share(deps, info, bill_id),
        ExecuteMsg::Settle { bill_id } => execute_settle(deps, bill_id),
        ExecuteMsg::Cancel { bill_id } => execute_cancel(deps, info, bill_id),
    }
}

fn execute_create_bill(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    participants: Vec<String>,
    share_amount: Uint128,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    if participants.is_empty() {
        return Err(ContractError::EmptyParticipants {});
    }
    if share_amount.is_zero() {
        return Err(ContractError::InvalidShareAmount {});
    }

    let recipient = deps.api.addr_validate(&recipient)?;
    let participants = validate_participants(deps.as_ref(), participants)?;
    let participant_count = Uint128::from(participants.len() as u128);
    let total_amount = share_amount
        .checked_mul(participant_count)
        .map_err(StdError::overflow)?;
    let mut config = CONFIG.load(deps.storage)?;
    let bill_id = config.next_bill_id;

    BILLS.save(
        deps.storage,
        bill_id,
        &Bill {
            id: bill_id,
            creator: info.sender.clone(),
            recipient: recipient.clone(),
            participants: participants.clone(),
            share_amount,
            total_amount,
            paid_count: 0,
            status: BillStatus::Open,
            memo,
        },
    )?;
    config.next_bill_id = config
        .next_bill_id
        .checked_add(1)
        .ok_or_else(|| StdError::generic_err("bill id overflow"))?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "create_bill")
        .add_attribute("bill_id", bill_id.to_string())
        .add_attribute("creator", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("participants", participants.len().to_string())
        .add_attribute("share_amount", share_amount)
        .add_attribute("total_amount", total_amount))
}

fn execute_pay_share(
    deps: DepsMut,
    info: MessageInfo,
    bill_id: u64,
) -> Result<Response, ContractError> {
    let mut bill = BILLS.load(deps.storage, bill_id)?;
    ensure_open(&bill)?;
    if !bill
        .participants
        .iter()
        .any(|participant| participant == &info.sender)
    {
        return Err(ContractError::NotParticipant {});
    }
    if PAID
        .may_load(deps.storage, (bill_id, &info.sender))?
        .unwrap_or_default()
    {
        return Err(ContractError::AlreadyPaid {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    require_payment(&info.funds, &config.denom, bill.share_amount)?;
    PAID.save(deps.storage, (bill_id, &info.sender), &true)?;
    bill.paid_count = bill
        .paid_count
        .checked_add(1)
        .ok_or_else(|| StdError::generic_err("paid count overflow"))?;
    BILLS.save(deps.storage, bill_id, &bill)?;
    config.balance = config
        .balance
        .checked_add(bill.share_amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "pay_share")
        .add_attribute("bill_id", bill_id.to_string())
        .add_attribute("participant", info.sender)
        .add_attribute("amount", bill.share_amount)
        .add_attribute("paid_count", bill.paid_count.to_string()))
}

fn execute_settle(deps: DepsMut, bill_id: u64) -> Result<Response, ContractError> {
    let mut bill = BILLS.load(deps.storage, bill_id)?;
    ensure_open(&bill)?;
    if bill.paid_count != bill.participants.len() as u64 {
        return Err(ContractError::BillNotFullyPaid {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    if config.balance < bill.total_amount {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: bill.total_amount,
        });
    }
    config.balance = config
        .balance
        .checked_sub(bill.total_amount)
        .map_err(StdError::overflow)?;
    bill.status = BillStatus::Settled;
    CONFIG.save(deps.storage, &config)?;
    BILLS.save(deps.storage, bill_id, &bill)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: bill.recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: bill.total_amount,
            }],
        })
        .add_attribute("action", "settle")
        .add_attribute("bill_id", bill_id.to_string())
        .add_attribute("recipient", bill.recipient)
        .add_attribute("amount", bill.total_amount)
        .add_attribute("balance", config.balance))
}

fn execute_cancel(
    deps: DepsMut,
    info: MessageInfo,
    bill_id: u64,
) -> Result<Response, ContractError> {
    let mut bill = BILLS.load(deps.storage, bill_id)?;
    ensure_open(&bill)?;
    if info.sender != bill.creator {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    let mut response = Response::new()
        .add_attribute("action", "cancel")
        .add_attribute("bill_id", bill_id.to_string());
    let mut refunded = Uint128::zero();
    for participant in &bill.participants {
        if PAID
            .may_load(deps.storage, (bill_id, participant))?
            .unwrap_or_default()
        {
            refunded = refunded
                .checked_add(bill.share_amount)
                .map_err(StdError::overflow)?;
            response = response.add_message(BankMsg::Send {
                to_address: participant.to_string(),
                amount: vec![Coin {
                    denom: config.denom.clone(),
                    amount: bill.share_amount,
                }],
            });
        }
    }

    if config.balance < refunded {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: refunded,
        });
    }
    config.balance = config
        .balance
        .checked_sub(refunded)
        .map_err(StdError::overflow)?;
    bill.status = BillStatus::Canceled;
    CONFIG.save(deps.storage, &config)?;
    BILLS.save(deps.storage, bill_id, &bill)?;

    Ok(response
        .add_attribute("refunded", refunded)
        .add_attribute("balance", config.balance))
}

fn validate_participants(
    deps: Deps,
    participants: Vec<String>,
) -> Result<Vec<cosmwasm_std::Addr>, ContractError> {
    let mut validated = Vec::with_capacity(participants.len());
    for participant in participants {
        let addr = deps.api.addr_validate(&participant)?;
        if validated.iter().any(|existing| existing == &addr) {
            return Err(ContractError::DuplicateParticipant {});
        }
        validated.push(addr);
    }
    Ok(validated)
}

fn ensure_open(bill: &Bill) -> Result<(), ContractError> {
    if bill.status != BillStatus::Open {
        return Err(ContractError::BillNotOpen {});
    }
    Ok(())
}

fn require_payment(
    funds: &[Coin],
    expected_denom: &str,
    required: Uint128,
) -> Result<(), ContractError> {
    match funds {
        [] => Err(ContractError::MissingPayment {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount != required {
                return Err(ContractError::IncorrectPayment {
                    paid: coin.amount,
                    required,
                });
            }
            Ok(())
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Bill { bill_id } => to_binary(&query_bill(deps, bill_id)?),
        QueryMsg::ShareStatus {
            bill_id,
            participant,
        } => to_binary(&query_share_status(deps, bill_id, participant)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        next_bill_id: config.next_bill_id,
        balance: config.balance,
    })
}

fn query_bill(deps: Deps, bill_id: u64) -> StdResult<BillResponse> {
    let bill = BILLS.load(deps.storage, bill_id)?;
    Ok(BillResponse {
        id: bill.id,
        creator: bill.creator.to_string(),
        recipient: bill.recipient.to_string(),
        participants: bill.participants.iter().map(ToString::to_string).collect(),
        share_amount: bill.share_amount,
        total_amount: bill.total_amount,
        paid_count: bill.paid_count,
        status: status_label(&bill.status).to_string(),
        memo: bill.memo,
    })
}

fn query_share_status(
    deps: Deps,
    bill_id: u64,
    participant: String,
) -> StdResult<ShareStatusResponse> {
    let participant_addr = deps.api.addr_validate(&participant)?;
    Ok(ShareStatusResponse {
        bill_id,
        participant,
        paid: PAID
            .may_load(deps.storage, (bill_id, &participant_addr))?
            .unwrap_or_default(),
    })
}

fn status_label(status: &BillStatus) -> &'static str {
    match status {
        BillStatus::Open => "open",
        BillStatus::Settled => "settled",
        BillStatus::Canceled => "canceled",
    }
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

    fn create_bill(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateBill {
                recipient: "merchant".to_string(),
                participants: vec!["alice".to_string(), "bob".to_string()],
                share_amount: Uint128::new(25),
                memo: Some("dinner".to_string()),
            },
        )
        .unwrap();
    }

    #[test]
    fn participants_pay_and_bill_settles() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_bill(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::PayShare { bill_id: 1 },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &coins(25, "uatom")),
            ExecuteMsg::PayShare { bill_id: 1 },
        )
        .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("anyone", &[]),
            ExecuteMsg::Settle { bill_id: 1 },
        )
        .unwrap();
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "merchant");
                assert_eq!(amount, &coins(50, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let bill: BillResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Bill { bill_id: 1 }).unwrap())
                .unwrap();
        assert_eq!(bill.status, "settled");
        assert_eq!(bill.paid_count, 2);
    }

    #[test]
    fn rejects_invalid_participants_and_payments() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let duplicate = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateBill {
                recipient: "merchant".to_string(),
                participants: vec!["alice".to_string(), "alice".to_string()],
                share_amount: Uint128::new(25),
                memo: None,
            },
        )
        .unwrap_err();
        assert!(matches!(duplicate, ContractError::DuplicateParticipant {}));

        create_bill(deps.as_mut());
        let non_participant = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("carol", &coins(25, "uatom")),
            ExecuteMsg::PayShare { bill_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(non_participant, ContractError::NotParticipant {}));

        let wrong_amount = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(24, "uatom")),
            ExecuteMsg::PayShare { bill_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(
            wrong_amount,
            ContractError::IncorrectPayment { .. }
        ));
    }

    #[test]
    fn cannot_settle_until_every_share_is_paid() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_bill(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::PayShare { bill_id: 1 },
        )
        .unwrap();

        let not_ready = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("anyone", &[]),
            ExecuteMsg::Settle { bill_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(not_ready, ContractError::BillNotFullyPaid {}));
    }

    #[test]
    fn creator_cancels_and_refunds_paid_participants() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_bill(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::PayShare { bill_id: 1 },
        )
        .unwrap();

        let unauthorized = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Cancel { bill_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::Unauthorized {}));

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Cancel { bill_id: 1 },
        )
        .unwrap();
        assert_eq!(response.messages.len(), 1);
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(25, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::zero());
    }
}
