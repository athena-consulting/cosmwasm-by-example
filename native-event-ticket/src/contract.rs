#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TicketsResponse};
use crate::state::{Config, CONFIG, PURCHASES};

const CONTRACT_NAME: &str = "crates.io:native-event-ticket";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: cosmwasm_std::Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let denom = msg.denom.trim().to_string();
    if denom.is_empty() {
        return Err(ContractError::EmptyDenom {});
    }
    if msg.price.is_zero() {
        return Err(ContractError::InvalidPrice {});
    }
    if msg.max_tickets == 0 {
        return Err(ContractError::InvalidQuantity {});
    }
    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };
    let organizer = deps.api.addr_validate(&msg.organizer)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            organizer: organizer.clone(),
            denom: denom.clone(),
            price: msg.price,
            max_tickets: msg.max_tickets,
            sold: 0,
            revenue: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("organizer", organizer)
        .add_attribute("denom", denom)
        .add_attribute("price", msg.price)
        .add_attribute("max_tickets", msg.max_tickets.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: cosmwasm_std::Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Buy { quantity } => execute_buy(deps, info, quantity),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
        ExecuteMsg::UpdateOrganizer { organizer } => {
            execute_update_organizer(deps, info, organizer)
        }
    }
}

fn execute_buy(deps: DepsMut, info: MessageInfo, quantity: u64) -> Result<Response, ContractError> {
    if quantity == 0 {
        return Err(ContractError::InvalidQuantity {});
    }
    let mut config = CONFIG.load(deps.storage)?;
    if config.sold + quantity > config.max_tickets {
        return Err(ContractError::SoldOut {});
    }
    let expected = config
        .price
        .checked_mul(Uint128::from(quantity as u128))
        .map_err(StdError::overflow)?;
    require_exact_payment(&info.funds, &config.denom, expected)?;

    config.sold += quantity;
    config.revenue = config
        .revenue
        .checked_add(expected)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    let current = PURCHASES
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();
    PURCHASES.save(deps.storage, &info.sender, &(current + quantity))?;

    Ok(Response::new()
        .add_attribute("action", "buy")
        .add_attribute("buyer", info.sender)
        .add_attribute("quantity", quantity.to_string())
        .add_attribute("denom", config.denom)
        .add_attribute("paid", expected)
        .add_attribute("sold", config.sold.to_string())
        .add_attribute("revenue", config.revenue))
}

fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    recipient: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let amount = amount.unwrap_or(config.revenue);
    if amount.is_zero() || amount > config.revenue {
        return Err(ContractError::InsufficientRevenue {
            available: config.revenue,
            requested: amount,
        });
    }
    let recipient = match recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => config.organizer.clone(),
    };
    config.revenue = config
        .revenue
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
        .add_attribute("revenue", config.revenue))
}

fn execute_update_organizer(
    deps: DepsMut,
    info: MessageInfo,
    organizer: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    config.organizer = deps.api.addr_validate(&organizer)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_organizer")
        .add_attribute("organizer", config.organizer))
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
                return Err(ContractError::PaymentMismatch {
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
pub fn query(deps: Deps, _env: cosmwasm_std::Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Tickets { buyer } => to_binary(&query_tickets(deps, buyer)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        organizer: config.organizer.to_string(),
        denom: config.denom,
        price: config.price,
        max_tickets: config.max_tickets,
        sold: config.sold,
        revenue: config.revenue,
    })
}

fn query_tickets(deps: Deps, buyer: String) -> StdResult<TicketsResponse> {
    let buyer_addr = deps.api.addr_validate(&buyer)?;
    let quantity = PURCHASES
        .may_load(deps.storage, &buyer_addr)?
        .unwrap_or_default();
    Ok(TicketsResponse { buyer, quantity })
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
                organizer: "organizer".to_string(),
                denom: "uatom".to_string(),
                price: Uint128::new(25),
                max_tickets: 3,
            },
        )
        .unwrap();
    }

    #[test]
    fn buy_records_tickets_and_revenue() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &coins(50, "uatom")),
            ExecuteMsg::Buy { quantity: 2 },
        )
        .unwrap();

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.sold, 2);
        assert_eq!(config.revenue, Uint128::new(50));

        let tickets: TicketsResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Tickets {
                    buyer: "buyer".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(tickets.quantity, 2);
    }

    #[test]
    fn rejects_wrong_payment_and_sold_out_purchase() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &coins(10, "uatom")),
            ExecuteMsg::Buy { quantity: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::PaymentMismatch { .. }));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &coins(100, "uatom")),
            ExecuteMsg::Buy { quantity: 4 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::SoldOut {}));
    }

    #[test]
    fn owner_withdraws_ticket_revenue() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &coins(75, "uatom")),
            ExecuteMsg::Buy { quantity: 3 },
        )
        .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: None,
                amount: Some(Uint128::new(40)),
            },
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "organizer");
                assert_eq!(amount, &coins(40, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.revenue, Uint128::new(35));
    }

    #[test]
    fn only_owner_can_withdraw_or_update_organizer() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &[]),
            ExecuteMsg::UpdateOrganizer {
                organizer: "new-organizer".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }
}
