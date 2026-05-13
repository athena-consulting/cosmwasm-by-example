#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{AttendeeResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Attendee, Config, ATTENDEES, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-rsvp-deposit";
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
    if msg.deposit_amount.is_zero() {
        return Err(ContractError::InvalidDeposit {});
    }
    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };
    let forfeiture_recipient = deps.api.addr_validate(&msg.forfeiture_recipient)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            forfeiture_recipient: forfeiture_recipient.clone(),
            denom: denom.clone(),
            deposit_amount: msg.deposit_amount,
            event_time: msg.event_time,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("forfeiture_recipient", forfeiture_recipient)
        .add_attribute("denom", denom)
        .add_attribute("deposit_amount", msg.deposit_amount)
        .add_attribute("event_time", msg.event_time.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Reserve {} => execute_reserve(deps, env, info),
        ExecuteMsg::CheckIn { attendee } => execute_check_in(deps, info, attendee),
        ExecuteMsg::Refund {} => execute_refund(deps, info),
        ExecuteMsg::Forfeit { attendee } => execute_forfeit(deps, env, info, attendee),
    }
}

fn execute_reserve(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if env.block.time.seconds() >= config.event_time {
        return Err(ContractError::EventClosed {});
    }
    if ATTENDEES.may_load(deps.storage, &info.sender)?.is_some() {
        return Err(ContractError::AlreadyReserved {});
    }
    require_exact_payment(&info.funds, &config.denom, config.deposit_amount)?;

    ATTENDEES.save(
        deps.storage,
        &info.sender,
        &Attendee {
            checked_in: false,
            settled: false,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "reserve")
        .add_attribute("attendee", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("deposit", config.deposit_amount))
}

fn execute_check_in(
    deps: DepsMut,
    info: MessageInfo,
    attendee: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let attendee_addr = deps.api.addr_validate(&attendee)?;
    let mut attendee_state = ATTENDEES
        .may_load(deps.storage, &attendee_addr)?
        .ok_or(ContractError::NotReserved {})?;
    if attendee_state.settled {
        return Err(ContractError::AlreadySettled {});
    }

    attendee_state.checked_in = true;
    ATTENDEES.save(deps.storage, &attendee_addr, &attendee_state)?;

    Ok(Response::new()
        .add_attribute("action", "check_in")
        .add_attribute("attendee", attendee_addr))
}

fn execute_refund(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut attendee = ATTENDEES
        .may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::NotReserved {})?;
    if attendee.settled {
        return Err(ContractError::AlreadySettled {});
    }
    if !attendee.checked_in {
        return Err(ContractError::NotCheckedIn {});
    }

    attendee.settled = true;
    ATTENDEES.save(deps.storage, &info.sender, &attendee)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: config.deposit_amount,
            }],
        })
        .add_attribute("action", "refund")
        .add_attribute("attendee", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", config.deposit_amount))
}

fn execute_forfeit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    attendee: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time.seconds() < config.event_time {
        return Err(ContractError::TooEarlyForForfeit {});
    }
    let attendee_addr = deps.api.addr_validate(&attendee)?;
    let mut attendee_state = ATTENDEES
        .may_load(deps.storage, &attendee_addr)?
        .ok_or(ContractError::NotReserved {})?;
    if attendee_state.settled {
        return Err(ContractError::AlreadySettled {});
    }
    if attendee_state.checked_in {
        return Err(ContractError::NotCheckedIn {});
    }

    attendee_state.settled = true;
    ATTENDEES.save(deps.storage, &attendee_addr, &attendee_state)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: config.forfeiture_recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: config.deposit_amount,
            }],
        })
        .add_attribute("action", "forfeit")
        .add_attribute("attendee", attendee_addr)
        .add_attribute("recipient", config.forfeiture_recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", config.deposit_amount))
}

fn require_exact_payment(
    funds: &[Coin],
    expected_denom: &str,
    expected_amount: Uint128,
) -> Result<(), ContractError> {
    match funds {
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount != expected_amount {
                return Err(ContractError::WrongDeposit {
                    expected: expected_amount,
                    received: coin.amount,
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
        QueryMsg::Attendee { attendee } => to_binary(&query_attendee(deps, attendee)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        forfeiture_recipient: config.forfeiture_recipient.to_string(),
        denom: config.denom,
        deposit_amount: config.deposit_amount,
        event_time: config.event_time,
    })
}

fn query_attendee(deps: Deps, attendee: String) -> StdResult<AttendeeResponse> {
    let attendee_addr = deps.api.addr_validate(&attendee)?;
    let attendee_state = ATTENDEES.may_load(deps.storage, &attendee_addr)?;
    Ok(AttendeeResponse {
        attendee,
        reserved: attendee_state.is_some(),
        checked_in: attendee_state
            .as_ref()
            .map(|state| state.checked_in)
            .unwrap_or_default(),
        settled: attendee_state
            .map(|state| state.settled)
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, CosmosMsg, Timestamp};

    fn env_at(seconds: u64) -> Env {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(seconds);
        env
    }

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            env_at(100),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: Some("owner".to_string()),
                forfeiture_recipient: "treasury".to_string(),
                denom: "uatom".to_string(),
                deposit_amount: Uint128::new(50),
                event_time: 200,
            },
        )
        .unwrap();
    }

    fn reserve(deps: DepsMut, attendee: &str) {
        execute(
            deps,
            env_at(150),
            mock_info(attendee, &coins(50, "uatom")),
            ExecuteMsg::Reserve {},
        )
        .unwrap();
    }

    #[test]
    fn checked_in_attendee_can_refund_deposit() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        reserve(deps.as_mut(), "alice");

        execute(
            deps.as_mut(),
            env_at(190),
            mock_info("owner", &[]),
            ExecuteMsg::CheckIn {
                attendee: "alice".to_string(),
            },
        )
        .unwrap();

        let response = execute(
            deps.as_mut(),
            env_at(210),
            mock_info("alice", &[]),
            ExecuteMsg::Refund {},
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(50, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let attendee: AttendeeResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(210),
                QueryMsg::Attendee {
                    attendee: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(attendee.checked_in);
        assert!(attendee.settled);
    }

    #[test]
    fn owner_forfeits_no_show_after_event_time() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        reserve(deps.as_mut(), "bob");

        let response = execute(
            deps.as_mut(),
            env_at(250),
            mock_info("owner", &[]),
            ExecuteMsg::Forfeit {
                attendee: "bob".to_string(),
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
    }

    #[test]
    fn rejects_late_or_duplicate_reservations() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        reserve(deps.as_mut(), "alice");

        let err = execute(
            deps.as_mut(),
            env_at(150),
            mock_info("alice", &coins(50, "uatom")),
            ExecuteMsg::Reserve {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::AlreadyReserved {}));

        let err = execute(
            deps.as_mut(),
            env_at(250),
            mock_info("carol", &coins(50, "uatom")),
            ExecuteMsg::Reserve {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EventClosed {}));
    }

    #[test]
    fn owner_guards_check_in_and_forfeit() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        reserve(deps.as_mut(), "bob");

        let err = execute(
            deps.as_mut(),
            env_at(190),
            mock_info("intruder", &[]),
            ExecuteMsg::CheckIn {
                attendee: "bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        let err = execute(
            deps.as_mut(),
            env_at(190),
            mock_info("owner", &[]),
            ExecuteMsg::Forfeit {
                attendee: "bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::TooEarlyForForfeit {}));
    }
}
