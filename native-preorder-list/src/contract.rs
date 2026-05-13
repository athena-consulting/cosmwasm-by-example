#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, OrderResponse, QueryMsg};
use crate::state::{Config, Order, CONFIG, ORDERS};

const CONTRACT_NAME: &str = "crates.io:native-preorder-list";
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
    let item = msg.item.trim().to_string();
    if item.is_empty() {
        return Err(ContractError::EmptyItem {});
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
            item: item.clone(),
            unit_price: msg.unit_price,
            max_units: msg.max_units,
            sold_units: 0,
            balance: Uint128::zero(),
            next_order_id: 1,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("item", item)
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
        ExecuteMsg::PlaceOrder {
            quantity,
            recipient,
            note,
        } => execute_place_order(deps, info, quantity, recipient, note),
        ExecuteMsg::MarkFulfilled { order_id } => execute_mark_fulfilled(deps, info, order_id),
        ExecuteMsg::RefundOrder { order_id } => execute_refund_order(deps, info, order_id),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
    }
}

fn execute_place_order(
    deps: DepsMut,
    info: MessageInfo,
    quantity: u64,
    recipient: Option<String>,
    note: Option<String>,
) -> Result<Response, ContractError> {
    if quantity == 0 {
        return Err(ContractError::InvalidQuantity {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    let new_sold_units = config
        .sold_units
        .checked_add(quantity)
        .ok_or_else(|| StdError::generic_err("sold units overflow"))?;
    if let Some(max_units) = config.max_units {
        if new_sold_units > max_units {
            return Err(ContractError::SoldOut {});
        }
    }

    let required = config
        .unit_price
        .checked_mul(Uint128::from(quantity as u128))
        .map_err(StdError::overflow)?;
    let coin = require_payment(&info.funds, &config.denom, required)?;
    let recipient = match recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => info.sender.clone(),
    };
    let order_id = config.next_order_id;

    config.next_order_id += 1;
    config.sold_units = new_sold_units;
    config.balance = config
        .balance
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    ORDERS.save(
        deps.storage,
        order_id,
        &Order {
            buyer: info.sender.clone(),
            recipient: recipient.clone(),
            quantity,
            paid: coin.amount,
            note,
            fulfilled: false,
            refunded: false,
        },
    )?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "place_order")
        .add_attribute("order_id", order_id.to_string())
        .add_attribute("buyer", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("quantity", quantity.to_string())
        .add_attribute("amount", coin.amount)
        .add_attribute("balance", config.balance))
}

fn execute_mark_fulfilled(
    deps: DepsMut,
    info: MessageInfo,
    order_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    ORDERS.update(
        deps.storage,
        order_id,
        |order| -> Result<_, ContractError> {
            let mut order = order.ok_or(ContractError::OrderNotFound {})?;
            if order.refunded {
                return Err(ContractError::AlreadyRefunded {});
            }
            if order.fulfilled {
                return Err(ContractError::AlreadyFulfilled {});
            }
            order.fulfilled = true;
            Ok(order)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "mark_fulfilled")
        .add_attribute("order_id", order_id.to_string()))
}

fn execute_refund_order(
    deps: DepsMut,
    info: MessageInfo,
    order_id: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let mut order = ORDERS
        .may_load(deps.storage, order_id)?
        .ok_or(ContractError::OrderNotFound {})?;
    if order.fulfilled {
        return Err(ContractError::AlreadyFulfilled {});
    }
    if order.refunded {
        return Err(ContractError::AlreadyRefunded {});
    }
    if config.balance < order.paid {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: order.paid,
        });
    }

    config.balance = config
        .balance
        .checked_sub(order.paid)
        .map_err(StdError::overflow)?;
    order.refunded = true;
    CONFIG.save(deps.storage, &config)?;
    ORDERS.save(deps.storage, order_id, &order)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: order.buyer.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: order.paid,
            }],
        })
        .add_attribute("action", "refund_order")
        .add_attribute("order_id", order_id.to_string())
        .add_attribute("buyer", order.buyer)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", order.paid)
        .add_attribute("balance", config.balance))
}

fn execute_withdraw(
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
        QueryMsg::Order { order_id } => to_binary(&query_order(deps, order_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        item: config.item,
        unit_price: config.unit_price,
        max_units: config.max_units,
        sold_units: config.sold_units,
        balance: config.balance,
        next_order_id: config.next_order_id,
    })
}

fn query_order(deps: Deps, order_id: u64) -> StdResult<OrderResponse> {
    let order = ORDERS.load(deps.storage, order_id)?;
    Ok(OrderResponse {
        order_id,
        buyer: order.buyer.to_string(),
        recipient: order.recipient.to_string(),
        quantity: order.quantity,
        paid: order.paid,
        note: order.note,
        fulfilled: order.fulfilled,
        refunded: order.refunded,
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
                item: "limited poster".to_string(),
                unit_price: Uint128::new(40),
                max_units: Some(3),
            },
        )
        .unwrap();
    }

    #[test]
    fn buyer_places_preorder() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(80, "uatom")),
            ExecuteMsg::PlaceOrder {
                quantity: 2,
                recipient: Some("bob".to_string()),
                note: Some("ship together".to_string()),
            },
        )
        .unwrap();

        let order: OrderResponse = from_binary(
            &query(deps.as_ref(), mock_env(), QueryMsg::Order { order_id: 1 }).unwrap(),
        )
        .unwrap();
        assert_eq!(order.buyer, "alice");
        assert_eq!(order.recipient, "bob");
        assert_eq!(order.quantity, 2);

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.sold_units, 2);
        assert_eq!(config.balance, Uint128::new(80));
    }

    #[test]
    fn rejects_underpayment_and_sold_out_orders() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let underpaid = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(79, "uatom")),
            ExecuteMsg::PlaceOrder {
                quantity: 2,
                recipient: None,
                note: None,
            },
        )
        .unwrap_err();
        assert!(matches!(underpaid, ContractError::Underpaid { .. }));

        let sold_out = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(160, "uatom")),
            ExecuteMsg::PlaceOrder {
                quantity: 4,
                recipient: None,
                note: None,
            },
        )
        .unwrap_err();
        assert!(matches!(sold_out, ContractError::SoldOut {}));
    }

    #[test]
    fn owner_fulfills_order_and_prevents_refund() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(40, "uatom")),
            ExecuteMsg::PlaceOrder {
                quantity: 1,
                recipient: None,
                note: None,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::MarkFulfilled { order_id: 1 },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::RefundOrder { order_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::AlreadyFulfilled {}));
    }

    #[test]
    fn owner_refunds_unfulfilled_order_and_withdraws_balance() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(40, "uatom")),
            ExecuteMsg::PlaceOrder {
                quantity: 1,
                recipient: None,
                note: None,
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &coins(40, "uatom")),
            ExecuteMsg::PlaceOrder {
                quantity: 1,
                recipient: None,
                note: None,
            },
        )
        .unwrap();

        let refund = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::RefundOrder { order_id: 1 },
        )
        .unwrap();
        match &refund.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(40, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let withdraw = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(40),
            },
        )
        .unwrap();
        match &withdraw.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(40, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }
}
