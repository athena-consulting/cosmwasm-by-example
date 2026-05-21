#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, InventoryItemResponse, ItemCountResponse,
    ItemResponse, MigrateMsg, QueryMsg, UserItemsResponse,
};
use crate::state::{Config, InventoryItem, CONFIG, USER_ITEMS, USER_ITEM_COUNT, USER_NEXT_ID};

const CONTRACT_NAME: &str = "crates.io:inventory-tracker";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_ITEMS_PER_USER: u32 = 100;
const MAX_ITEMS_PER_USER: u32 = 1_000;
const MAX_QUERY_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let max_items_per_user = msg.max_items_per_user.unwrap_or(DEFAULT_MAX_ITEMS_PER_USER);
    validate_max_items(max_items_per_user)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender.clone(),
            max_items_per_user,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("max_items_per_user", max_items_per_user.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddItem {
            sku,
            name,
            quantity,
            note,
        } => add_item(deps, env, info, sku, name, quantity, note),
        ExecuteMsg::UpdateItem {
            item_id,
            sku,
            name,
            note,
        } => update_item(deps, env, info, item_id, sku, name, note),
        ExecuteMsg::SetQuantity { item_id, quantity } => {
            set_quantity(deps, env, info, item_id, quantity)
        }
        ExecuteMsg::IncreaseQuantity { item_id, amount } => {
            increase_quantity(deps, env, info, item_id, amount)
        }
        ExecuteMsg::DecreaseQuantity { item_id, amount } => {
            decrease_quantity(deps, env, info, item_id, amount)
        }
        ExecuteMsg::DeleteItem { item_id } => delete_item(deps, info, item_id),
        ExecuteMsg::UpdateConfig { max_items_per_user } => {
            update_config(deps, info, max_items_per_user)
        }
        ExecuteMsg::TransferOwnership { new_owner } => transfer_ownership(deps, info, new_owner),
    }
}

pub fn add_item(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sku: String,
    name: String,
    quantity: Uint128,
    note: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let count = USER_ITEM_COUNT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    if count >= config.max_items_per_user {
        return Err(ContractError::ItemLimitReached {});
    }

    validate_nonzero("quantity", quantity)?;
    let sku = validate_sku(sku)?;
    let name = validate_text("name", name, 3, 80)?;
    let note = validate_optional_text("note", note, 1, 160)?;

    let item_id = USER_NEXT_ID
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(1);
    let item = InventoryItem {
        item_id,
        owner: info.sender.clone(),
        sku,
        name,
        quantity,
        note,
        created_at_height: env.block.height,
        updated_at_height: env.block.height,
    };

    USER_ITEMS.save(deps.storage, (&info.sender, item_id), &item)?;
    USER_NEXT_ID.save(deps.storage, &info.sender, &(item_id + 1))?;
    USER_ITEM_COUNT.save(deps.storage, &info.sender, &(count + 1))?;

    Ok(Response::new()
        .add_attribute("action", "add_item")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string())
        .add_attribute("quantity", quantity.to_string()))
}

pub fn update_item(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    item_id: u64,
    sku: String,
    name: String,
    note: Option<String>,
) -> Result<Response, ContractError> {
    let sku = validate_sku(sku)?;
    let name = validate_text("name", name, 3, 80)?;
    let note = validate_optional_text("note", note, 1, 160)?;

    USER_ITEMS.update(
        deps.storage,
        (&info.sender, item_id),
        |item| -> Result<InventoryItem, ContractError> {
            let mut item = item.ok_or(ContractError::ItemNotFound {})?;
            item.sku = sku;
            item.name = name;
            item.note = note;
            item.updated_at_height = env.block.height;
            Ok(item)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "update_item")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string()))
}

pub fn set_quantity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    item_id: u64,
    quantity: Uint128,
) -> Result<Response, ContractError> {
    USER_ITEMS.update(
        deps.storage,
        (&info.sender, item_id),
        |item| -> Result<InventoryItem, ContractError> {
            let mut item = item.ok_or(ContractError::ItemNotFound {})?;
            item.quantity = quantity;
            item.updated_at_height = env.block.height;
            Ok(item)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "set_quantity")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string())
        .add_attribute("quantity", quantity.to_string()))
}

pub fn increase_quantity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    item_id: u64,
    amount: Uint128,
) -> Result<Response, ContractError> {
    validate_nonzero("amount", amount)?;

    let quantity = USER_ITEMS.update(
        deps.storage,
        (&info.sender, item_id),
        |item| -> Result<InventoryItem, ContractError> {
            let mut item = item.ok_or(ContractError::ItemNotFound {})?;
            item.quantity += amount;
            item.updated_at_height = env.block.height;
            Ok(item)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "increase_quantity")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string())
        .add_attribute("quantity", quantity.quantity.to_string()))
}

pub fn decrease_quantity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    item_id: u64,
    amount: Uint128,
) -> Result<Response, ContractError> {
    validate_nonzero("amount", amount)?;

    let quantity = USER_ITEMS.update(
        deps.storage,
        (&info.sender, item_id),
        |item| -> Result<InventoryItem, ContractError> {
            let mut item = item.ok_or(ContractError::ItemNotFound {})?;
            item.quantity = item
                .quantity
                .checked_sub(amount)
                .map_err(|_| ContractError::InsufficientQuantity {})?;
            item.updated_at_height = env.block.height;
            Ok(item)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "decrease_quantity")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string())
        .add_attribute("quantity", quantity.quantity.to_string()))
}

pub fn delete_item(
    deps: DepsMut,
    info: MessageInfo,
    item_id: u64,
) -> Result<Response, ContractError> {
    if !USER_ITEMS.has(deps.storage, (&info.sender, item_id)) {
        return Err(ContractError::ItemNotFound {});
    }

    USER_ITEMS.remove(deps.storage, (&info.sender, item_id));
    let count = USER_ITEM_COUNT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default()
        .saturating_sub(1);
    USER_ITEM_COUNT.save(deps.storage, &info.sender, &count)?;

    Ok(Response::new()
        .add_attribute("action", "delete_item")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string()))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    max_items_per_user: u32,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;
    validate_max_items(max_items_per_user)?;

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.max_items_per_user = max_items_per_user;
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("max_items_per_user", max_items_per_user.to_string()))
}

pub fn transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;
    let new_owner = deps.api.addr_validate(&new_owner)?;

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.owner = new_owner.clone();
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_attribute("new_owner", new_owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Item { owner, item_id } => to_json_binary(&query_item(deps, owner, item_id)?),
        QueryMsg::UserItems {
            owner,
            start_after,
            limit,
        } => to_json_binary(&query_user_items(deps, owner, start_after, limit)?),
        QueryMsg::ItemCount { owner } => to_json_binary(&query_item_count(deps, owner)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "migrate"))
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        max_items_per_user: config.max_items_per_user,
    })
}

fn query_item(deps: Deps, owner: String, item_id: u64) -> StdResult<ItemResponse> {
    let owner = deps.api.addr_validate(&owner)?;
    let item = USER_ITEMS.load(deps.storage, (&owner, item_id))?;
    Ok(ItemResponse {
        item: item_to_response(item),
    })
}

fn query_user_items(
    deps: Deps,
    owner: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<UserItemsResponse> {
    let owner = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(20).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let items = USER_ITEMS
        .prefix(&owner)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, inventory_item)| item_to_response(inventory_item)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(UserItemsResponse {
        owner: owner.to_string(),
        items,
    })
}

fn query_item_count(deps: Deps, owner: String) -> StdResult<ItemCountResponse> {
    let owner = deps.api.addr_validate(&owner)?;
    let count = USER_ITEM_COUNT
        .may_load(deps.storage, &owner)?
        .unwrap_or_default();

    Ok(ItemCountResponse {
        owner: owner.to_string(),
        count,
    })
}

fn assert_owner(deps: Deps, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != *sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn item_to_response(item: InventoryItem) -> InventoryItemResponse {
    InventoryItemResponse {
        item_id: item.item_id,
        owner: item.owner.to_string(),
        sku: item.sku,
        name: item.name,
        quantity: item.quantity,
        note: item.note,
        created_at_height: item.created_at_height,
        updated_at_height: item.updated_at_height,
    }
}

fn validate_max_items(value: u32) -> Result<(), ContractError> {
    if value == 0 || value > MAX_ITEMS_PER_USER {
        return Err(ContractError::InvalidInput {
            reason: format!("max_items_per_user must be between 1 and {MAX_ITEMS_PER_USER}"),
        });
    }
    Ok(())
}

fn validate_nonzero(field: &str, value: Uint128) -> Result<(), ContractError> {
    if value.is_zero() {
        return Err(ContractError::InvalidInput {
            reason: format!("{field} must be greater than zero"),
        });
    }
    Ok(())
}

fn validate_sku(value: String) -> Result<String, ContractError> {
    let value = validate_text("sku", value, 2, 32)?;
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(ContractError::InvalidInput {
            reason: "sku can only contain ASCII letters, numbers, hyphens, or underscores"
                .to_string(),
        });
    }
    Ok(value)
}

fn validate_text(
    field: &str,
    value: String,
    min_len: usize,
    max_len: usize,
) -> Result<String, ContractError> {
    let value = value.trim().to_string();
    if value.len() < min_len || value.len() > max_len {
        return Err(ContractError::InvalidInput {
            reason: format!("{field} must be {min_len} to {max_len} characters"),
        });
    }
    Ok(value)
}

fn validate_optional_text(
    field: &str,
    value: Option<String>,
    min_len: usize,
    max_len: usize,
) -> Result<Option<String>, ContractError> {
    value
        .map(|value| validate_text(field, value, min_len, max_len))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env, mock_info},
    };

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {
                max_items_per_user: Some(2),
            },
        )
        .unwrap();
    }

    fn add_sample_item(deps: DepsMut, sender: &str, sku: &str, quantity: u128) {
        execute(
            deps,
            mock_env(),
            mock_info(sender, &[]),
            ExecuteMsg::AddItem {
                sku: sku.to_string(),
                name: "Hardware Wallet".to_string(),
                quantity: Uint128::new(quantity),
                note: Some("Cold storage".to_string()),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_and_query_config() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.owner, "admin");
        assert_eq!(config.max_items_per_user, 2);
    }

    #[test]
    fn user_adds_and_lists_items() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "HW-1", 2);
        add_sample_item(deps.as_mut(), "alice", "CABLE-1", 5);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::UserItems {
                owner: "alice".to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let items: UserItemsResponse = from_json(&res).unwrap();

        assert_eq!(items.owner, "alice");
        assert_eq!(items.items.len(), 2);
        assert_eq!(items.items[0].item_id, 1);
        assert_eq!(items.items[1].sku, "CABLE-1");
    }

    #[test]
    fn user_updates_metadata_and_quantity() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "HW-1", 2);

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateItem {
                item_id: 1,
                sku: "HW-PRO".to_string(),
                name: "Hardware Wallet Pro".to_string(),
                note: None,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::IncreaseQuantity {
                item_id: 1,
                amount: Uint128::new(3),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::DecreaseQuantity {
                item_id: 1,
                amount: Uint128::new(1),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Item {
                owner: "alice".to_string(),
                item_id: 1,
            },
        )
        .unwrap();
        let item: ItemResponse = from_json(&res).unwrap();

        assert_eq!(item.item.sku, "HW-PRO");
        assert_eq!(item.item.name, "Hardware Wallet Pro");
        assert_eq!(item.item.quantity, Uint128::new(4));
        assert_eq!(item.item.note, None);
    }

    #[test]
    fn decrease_cannot_underflow() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "HW-1", 2);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::DecreaseQuantity {
                item_id: 1,
                amount: Uint128::new(3),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::InsufficientQuantity {});
    }

    #[test]
    fn users_cannot_modify_each_others_items() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "HW-1", 2);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::SetQuantity {
                item_id: 1,
                quantity: Uint128::new(10),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::ItemNotFound {});
    }

    #[test]
    fn item_limit_and_validation_are_enforced() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "HW-1", 2);
        add_sample_item(deps.as_mut(), "alice", "CABLE-1", 5);

        let limit_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::AddItem {
                sku: "THIRD".to_string(),
                name: "Third Item".to_string(),
                quantity: Uint128::new(1),
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(limit_err, ContractError::ItemLimitReached {});

        let sku_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::AddItem {
                sku: "bad sku".to_string(),
                name: "Invalid Sku".to_string(),
                quantity: Uint128::new(1),
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            sku_err,
            ContractError::InvalidInput {
                reason: "sku can only contain ASCII letters, numbers, hyphens, or underscores"
                    .to_string()
            }
        );

        let zero_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::AddItem {
                sku: "ZERO".to_string(),
                name: "Zero Quantity".to_string(),
                quantity: Uint128::zero(),
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            zero_err,
            ContractError::InvalidInput {
                reason: "quantity must be greater than zero".to_string()
            }
        );
    }

    #[test]
    fn delete_updates_count() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "HW-1", 2);

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::DeleteItem { item_id: 1 },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ItemCount {
                owner: "alice".to_string(),
            },
        )
        .unwrap();
        let count: ItemCountResponse = from_json(&res).unwrap();

        assert_eq!(count.count, 0);
    }

    #[test]
    fn owner_updates_config_and_transfers_ownership() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let unauthorized = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateConfig {
                max_items_per_user: 3,
            },
        )
        .unwrap_err();
        assert_eq!(unauthorized, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                max_items_per_user: 3,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::TransferOwnership {
                new_owner: "new_admin".to_string(),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.owner, "new_admin");
        assert_eq!(config.max_items_per_user, 3);
    }
}
