#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, ItemCountResponse, ItemResponse, MigrateMsg,
    QueryMsg, UserItemsResponse, WishItemResponse,
};
use crate::state::{Config, WishItem, CONFIG, USER_ITEMS, USER_ITEM_COUNT, USER_NEXT_ID};

const CONTRACT_NAME: &str = "crates.io:wishlist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_ITEMS_PER_USER: u32 = 50;
const MAX_ITEMS_PER_USER: u32 = 500;
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
            title,
            description,
            url,
            priority,
        } => add_item(deps, env, info, title, description, url, priority),
        ExecuteMsg::UpdateItem {
            item_id,
            title,
            description,
            url,
            priority,
        } => update_item(deps, env, info, item_id, title, description, url, priority),
        ExecuteMsg::SetPurchased { item_id, purchased } => {
            set_purchased(deps, env, info, item_id, purchased)
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
    title: String,
    description: Option<String>,
    url: Option<String>,
    priority: u8,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let count = USER_ITEM_COUNT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    if count >= config.max_items_per_user {
        return Err(ContractError::ItemLimitReached {});
    }

    let title = validate_text("title", title, 3, 80)?;
    let description = validate_optional_text("description", description, 1, 200)?;
    let url = validate_optional_url(url)?;
    validate_priority(priority)?;

    let item_id = USER_NEXT_ID
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(1);
    let item = WishItem {
        item_id,
        owner: info.sender.clone(),
        title,
        description,
        url,
        priority,
        purchased: false,
        created_at_height: env.block.height,
        updated_at_height: env.block.height,
    };

    USER_ITEMS.save(deps.storage, (&info.sender, item_id), &item)?;
    USER_NEXT_ID.save(deps.storage, &info.sender, &(item_id + 1))?;
    USER_ITEM_COUNT.save(deps.storage, &info.sender, &(count + 1))?;

    Ok(Response::new()
        .add_attribute("action", "add_item")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string()))
}

#[allow(clippy::too_many_arguments)]
pub fn update_item(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    item_id: u64,
    title: String,
    description: Option<String>,
    url: Option<String>,
    priority: u8,
) -> Result<Response, ContractError> {
    let title = validate_text("title", title, 3, 80)?;
    let description = validate_optional_text("description", description, 1, 200)?;
    let url = validate_optional_url(url)?;
    validate_priority(priority)?;

    USER_ITEMS.update(
        deps.storage,
        (&info.sender, item_id),
        |item| -> Result<WishItem, ContractError> {
            let mut item = item.ok_or(ContractError::ItemNotFound {})?;
            item.title = title;
            item.description = description;
            item.url = url;
            item.priority = priority;
            item.updated_at_height = env.block.height;
            Ok(item)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "update_item")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string()))
}

pub fn set_purchased(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    item_id: u64,
    purchased: bool,
) -> Result<Response, ContractError> {
    USER_ITEMS.update(
        deps.storage,
        (&info.sender, item_id),
        |item| -> Result<WishItem, ContractError> {
            let mut item = item.ok_or(ContractError::ItemNotFound {})?;
            item.purchased = purchased;
            item.updated_at_height = env.block.height;
            Ok(item)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "set_purchased")
        .add_attribute("owner", info.sender)
        .add_attribute("item_id", item_id.to_string())
        .add_attribute("purchased", purchased.to_string()))
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
        .map(|item| item.map(|(_, wish)| item_to_response(wish)))
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

fn item_to_response(item: WishItem) -> WishItemResponse {
    WishItemResponse {
        item_id: item.item_id,
        owner: item.owner.to_string(),
        title: item.title,
        description: item.description,
        url: item.url,
        priority: item.priority,
        purchased: item.purchased,
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

fn validate_priority(value: u8) -> Result<(), ContractError> {
    if !(1..=5).contains(&value) {
        return Err(ContractError::InvalidInput {
            reason: "priority must be between 1 and 5".to_string(),
        });
    }
    Ok(())
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

fn validate_optional_url(value: Option<String>) -> Result<Option<String>, ContractError> {
    value
        .map(|value| {
            let value = validate_text("url", value, 8, 200)?;
            if !value.starts_with("https://") {
                return Err(ContractError::InvalidInput {
                    reason: "url must start with https://".to_string(),
                });
            }
            Ok(value)
        })
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

    fn add_sample_item(deps: DepsMut, sender: &str, title: &str, priority: u8) {
        execute(
            deps,
            mock_env(),
            mock_info(sender, &[]),
            ExecuteMsg::AddItem {
                title: title.to_string(),
                description: Some("Useful item".to_string()),
                url: Some("https://example.com/item".to_string()),
                priority,
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
        add_sample_item(deps.as_mut(), "alice", "Hardware Wallet", 5);
        add_sample_item(deps.as_mut(), "alice", "Desk Lamp", 2);

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
        assert_eq!(items.items[1].title, "Desk Lamp");
    }

    #[test]
    fn user_updates_and_marks_item_purchased() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "Hardware Wallet", 5);

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateItem {
                item_id: 1,
                title: "Hardware Wallet Pro".to_string(),
                description: None,
                url: None,
                priority: 4,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SetPurchased {
                item_id: 1,
                purchased: true,
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

        assert_eq!(item.item.title, "Hardware Wallet Pro");
        assert_eq!(item.item.priority, 4);
        assert!(item.item.purchased);
    }

    #[test]
    fn users_cannot_modify_each_others_items() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "Hardware Wallet", 5);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::SetPurchased {
                item_id: 1,
                purchased: true,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::ItemNotFound {});
    }

    #[test]
    fn item_limit_and_validation_are_enforced() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        add_sample_item(deps.as_mut(), "alice", "Hardware Wallet", 5);
        add_sample_item(deps.as_mut(), "alice", "Desk Lamp", 2);

        let limit_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::AddItem {
                title: "Third Item".to_string(),
                description: None,
                url: None,
                priority: 3,
            },
        )
        .unwrap_err();
        assert_eq!(limit_err, ContractError::ItemLimitReached {});

        let priority_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::AddItem {
                title: "Invalid Priority".to_string(),
                description: None,
                url: None,
                priority: 6,
            },
        )
        .unwrap_err();
        assert_eq!(
            priority_err,
            ContractError::InvalidInput {
                reason: "priority must be between 1 and 5".to_string()
            }
        );
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
