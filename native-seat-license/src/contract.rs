#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SeatResponse};
use crate::state::{Config, CONFIG, SEATS};

const CONTRACT_NAME: &str = "crates.io:native-seat-license";
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
    if msg.seat_price.is_zero() {
        return Err(ContractError::InvalidSeatPrice {});
    }
    if msg.max_seats == 0 {
        return Err(ContractError::InvalidMaxSeats {});
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
            seat_price: msg.seat_price,
            max_seats: msg.max_seats,
            sold_seats: 0,
            balance: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("seat_price", msg.seat_price)
        .add_attribute("max_seats", msg.max_seats.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuySeat { account } => execute_buy_seat(deps, info, account),
        ExecuteMsg::TransferSeat { recipient } => execute_transfer_seat(deps, info, recipient),
        ExecuteMsg::RevokeSeat { account } => execute_revoke_seat(deps, info, account),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
    }
}

fn execute_buy_seat(
    deps: DepsMut,
    info: MessageInfo,
    account: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.sold_seats >= config.max_seats {
        return Err(ContractError::SoldOut {});
    }
    require_payment(&info.funds, &config.denom, config.seat_price)?;
    let account = match account {
        Some(account) => deps.api.addr_validate(&account)?,
        None => info.sender.clone(),
    };
    if SEATS.may_load(deps.storage, &account)?.unwrap_or_default() {
        return Err(ContractError::SeatAlreadyOwned {});
    }

    SEATS.save(deps.storage, &account, &true)?;
    config.sold_seats = config
        .sold_seats
        .checked_add(1)
        .ok_or_else(|| StdError::generic_err("sold seats overflow"))?;
    config.balance = config
        .balance
        .checked_add(config.seat_price)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "buy_seat")
        .add_attribute("payer", info.sender)
        .add_attribute("account", account)
        .add_attribute("sold_seats", config.sold_seats.to_string())
        .add_attribute("balance", config.balance))
}

fn execute_transfer_seat(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, ContractError> {
    let recipient = deps.api.addr_validate(&recipient)?;
    if !SEATS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default()
    {
        return Err(ContractError::SeatNotOwned {});
    }
    if SEATS
        .may_load(deps.storage, &recipient)?
        .unwrap_or_default()
    {
        return Err(ContractError::SeatAlreadyOwned {});
    }

    SEATS.remove(deps.storage, &info.sender);
    SEATS.save(deps.storage, &recipient, &true)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_seat")
        .add_attribute("from", info.sender)
        .add_attribute("recipient", recipient))
}

fn execute_revoke_seat(
    deps: DepsMut,
    info: MessageInfo,
    account: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let account = deps.api.addr_validate(&account)?;
    if !SEATS.may_load(deps.storage, &account)?.unwrap_or_default() {
        return Err(ContractError::SeatNotOwned {});
    }

    SEATS.remove(deps.storage, &account);
    config.sold_seats = config
        .sold_seats
        .checked_sub(1)
        .ok_or_else(|| StdError::generic_err("sold seats underflow"))?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "revoke_seat")
        .add_attribute("account", account)
        .add_attribute("sold_seats", config.sold_seats.to_string()))
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Seat { account } => to_binary(&query_seat(deps, account)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        seat_price: config.seat_price,
        max_seats: config.max_seats,
        sold_seats: config.sold_seats,
        balance: config.balance,
    })
}

fn query_seat(deps: Deps, account: String) -> StdResult<SeatResponse> {
    let account_addr = deps.api.addr_validate(&account)?;
    Ok(SeatResponse {
        account,
        has_seat: SEATS
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
                seat_price: Uint128::new(25),
                max_seats: 2,
            },
        )
        .unwrap();
    }

    #[test]
    fn buys_and_transfers_seat() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::BuySeat { account: None },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::TransferSeat {
                recipient: "bob".to_string(),
            },
        )
        .unwrap();

        let alice: SeatResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Seat {
                    account: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let bob: SeatResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Seat {
                    account: "bob".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(!alice.has_seat);
        assert!(bob.has_seat);
    }

    #[test]
    fn enforces_supply_and_duplicate_seats() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::BuySeat { account: None },
        )
        .unwrap();
        let duplicate = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::BuySeat { account: None },
        )
        .unwrap_err();
        assert!(matches!(duplicate, ContractError::SeatAlreadyOwned {}));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &coins(25, "uatom")),
            ExecuteMsg::BuySeat { account: None },
        )
        .unwrap();
        let sold_out = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("carol", &coins(25, "uatom")),
            ExecuteMsg::BuySeat { account: None },
        )
        .unwrap_err();
        assert!(matches!(sold_out, ContractError::SoldOut {}));
    }

    #[test]
    fn owner_revokes_seat_to_free_supply() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::BuySeat { account: None },
        )
        .unwrap();
        let unauthorized = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::RevokeSeat {
                account: "alice".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::Unauthorized {}));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::RevokeSeat {
                account: "alice".to_string(),
            },
        )
        .unwrap();
        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.sold_seats, 0);
    }

    #[test]
    fn owner_withdraws_collected_fees() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(25, "uatom")),
            ExecuteMsg::BuySeat { account: None },
        )
        .unwrap();
        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(10),
            },
        )
        .unwrap();
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(10, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }
}
