#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SlotResponse};
use crate::state::{Config, Slot, CONFIG, SLOTS};

const CONTRACT_NAME: &str = "crates.io:native-billboard-rental";
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
    if msg.slot_price.is_zero() {
        return Err(ContractError::InvalidSlotPrice {});
    }
    if msg.rental_seconds == 0 {
        return Err(ContractError::InvalidRentalDuration {});
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
            slot_price: msg.slot_price,
            rental_seconds: msg.rental_seconds,
            next_slot_id: 1,
            balance: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("slot_price", msg.slot_price)
        .add_attribute("rental_seconds", msg.rental_seconds.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateSlot { label } => execute_create_slot(deps, info, label),
        ExecuteMsg::RentSlot { slot_id, message } => {
            execute_rent_slot(deps, env, info, slot_id, message)
        }
        ExecuteMsg::UpdateMessage { slot_id, message } => {
            execute_update_message(deps, env, info, slot_id, message)
        }
        ExecuteMsg::CloseSlot { slot_id } => execute_close_slot(deps, info, slot_id),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
    }
}

fn execute_create_slot(
    deps: DepsMut,
    info: MessageInfo,
    label: String,
) -> Result<Response, ContractError> {
    let label = validate_text(label, ContractError::EmptyLabel {})?;
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let slot_id = config.next_slot_id;

    SLOTS.save(
        deps.storage,
        slot_id,
        &Slot {
            id: slot_id,
            label: label.clone(),
            active: true,
            renter: None,
            message: None,
            paid_until: 0,
        },
    )?;
    config.next_slot_id = config
        .next_slot_id
        .checked_add(1)
        .ok_or_else(|| StdError::generic_err("slot id overflow"))?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "create_slot")
        .add_attribute("slot_id", slot_id.to_string())
        .add_attribute("label", label))
}

fn execute_rent_slot(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    slot_id: u64,
    message: String,
) -> Result<Response, ContractError> {
    let message = validate_text(message, ContractError::EmptyMessage {})?;
    let mut slot = SLOTS.load(deps.storage, slot_id)?;
    ensure_available(&slot, &env)?;

    let mut config = CONFIG.load(deps.storage)?;
    require_payment(&info.funds, &config.denom, config.slot_price)?;
    slot.renter = Some(info.sender.clone());
    slot.message = Some(message);
    slot.paid_until = env
        .block
        .time
        .seconds()
        .checked_add(config.rental_seconds)
        .ok_or_else(|| StdError::generic_err("rental expiry overflow"))?;
    SLOTS.save(deps.storage, slot_id, &slot)?;
    config.balance = config
        .balance
        .checked_add(config.slot_price)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "rent_slot")
        .add_attribute("slot_id", slot_id.to_string())
        .add_attribute("renter", info.sender)
        .add_attribute("paid_until", slot.paid_until.to_string())
        .add_attribute("balance", config.balance))
}

fn execute_update_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    slot_id: u64,
    message: String,
) -> Result<Response, ContractError> {
    let message = validate_text(message, ContractError::EmptyMessage {})?;
    let mut slot = SLOTS.load(deps.storage, slot_id)?;
    if !slot.active {
        return Err(ContractError::SlotClosed {});
    }
    if env.block.time.seconds() >= slot.paid_until {
        return Err(ContractError::SlotNotAvailable {
            paid_until: slot.paid_until,
        });
    }
    if slot.renter.as_ref() != Some(&info.sender) {
        return Err(ContractError::NotRenter {});
    }

    slot.message = Some(message);
    SLOTS.save(deps.storage, slot_id, &slot)?;

    Ok(Response::new()
        .add_attribute("action", "update_message")
        .add_attribute("slot_id", slot_id.to_string())
        .add_attribute("renter", info.sender))
}

fn execute_close_slot(
    deps: DepsMut,
    info: MessageInfo,
    slot_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let mut slot = SLOTS.load(deps.storage, slot_id)?;
    slot.active = false;
    SLOTS.save(deps.storage, slot_id, &slot)?;

    Ok(Response::new()
        .add_attribute("action", "close_slot")
        .add_attribute("slot_id", slot_id.to_string()))
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

fn validate_text(value: String, error: ContractError) -> Result<String, ContractError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(error);
    }
    Ok(value)
}

fn ensure_available(slot: &Slot, env: &Env) -> Result<(), ContractError> {
    if !slot.active {
        return Err(ContractError::SlotClosed {});
    }
    if env.block.time.seconds() < slot.paid_until {
        return Err(ContractError::SlotNotAvailable {
            paid_until: slot.paid_until,
        });
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
            if coin.amount < required {
                return Err(ContractError::Underpaid {
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Slot { slot_id } => to_binary(&query_slot(deps, env, slot_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        slot_price: config.slot_price,
        rental_seconds: config.rental_seconds,
        next_slot_id: config.next_slot_id,
        balance: config.balance,
    })
}

fn query_slot(deps: Deps, env: Env, slot_id: u64) -> StdResult<SlotResponse> {
    let slot = SLOTS.load(deps.storage, slot_id)?;
    Ok(SlotResponse {
        id: slot.id,
        label: slot.label,
        active: slot.active,
        renter: slot.renter.map(|renter| renter.to_string()),
        message: slot.message,
        paid_until: slot.paid_until,
        is_available: slot.active && env.block.time.seconds() >= slot.paid_until,
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
                denom: "uatom".to_string(),
                slot_price: Uint128::new(25),
                rental_seconds: 100,
            },
        )
        .unwrap();
    }

    fn create_slot(deps: DepsMut) {
        execute(
            deps,
            env_at(100),
            mock_info("owner", &[]),
            ExecuteMsg::CreateSlot {
                label: "homepage".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn renter_rents_and_updates_slot() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_slot(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Visit Alice".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env_at(120),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateMessage {
                slot_id: 1,
                message: "Alice updated".to_string(),
            },
        )
        .unwrap();

        let slot: SlotResponse =
            from_binary(&query(deps.as_ref(), env_at(120), QueryMsg::Slot { slot_id: 1 }).unwrap())
                .unwrap();
        assert_eq!(slot.renter, Some("alice".to_string()));
        assert_eq!(slot.message, Some("Alice updated".to_string()));
        assert_eq!(slot.paid_until, 200);
        assert!(!slot.is_available);
    }

    #[test]
    fn prevents_renting_until_expiry() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_slot(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Alice".to_string(),
            },
        )
        .unwrap();
        let early = execute(
            deps.as_mut(),
            env_at(150),
            mock_info("bob", &coins(25, "uatom")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(early, ContractError::SlotNotAvailable { .. }));

        execute(
            deps.as_mut(),
            env_at(200),
            mock_info("bob", &coins(25, "uatom")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Bob".to_string(),
            },
        )
        .unwrap();
        let slot: SlotResponse =
            from_binary(&query(deps.as_ref(), env_at(200), QueryMsg::Slot { slot_id: 1 }).unwrap())
                .unwrap();
        assert_eq!(slot.renter, Some("bob".to_string()));
    }

    #[test]
    fn rejects_unauthorized_update_and_wrong_payment() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_slot(deps.as_mut());

        let wrong_denom = execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(25, "uosmo")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Alice".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(wrong_denom, ContractError::WrongDenom { .. }));

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Alice".to_string(),
            },
        )
        .unwrap();
        let unauthorized = execute(
            deps.as_mut(),
            env_at(120),
            mock_info("bob", &[]),
            ExecuteMsg::UpdateMessage {
                slot_id: 1,
                message: "Bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::NotRenter {}));
    }

    #[test]
    fn owner_closes_slot_and_withdraws_fees() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_slot(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Alice".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env_at(110),
            mock_info("owner", &[]),
            ExecuteMsg::CloseSlot { slot_id: 1 },
        )
        .unwrap();
        let closed = execute(
            deps.as_mut(),
            env_at(200),
            mock_info("bob", &coins(25, "uatom")),
            ExecuteMsg::RentSlot {
                slot_id: 1,
                message: "Bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(closed, ContractError::SlotClosed {}));

        let response = execute(
            deps.as_mut(),
            env_at(120),
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
            from_binary(&query(deps.as_ref(), env_at(120), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(5));
    }
}
