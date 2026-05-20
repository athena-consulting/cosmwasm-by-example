#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, ItemResponse, ItemsResponse, QueryMsg,
    RatingResponse,
};
use crate::state::{Config, RatedItem, CONFIG, ITEMS, RATINGS};

const CONTRACT_NAME: &str = "crates.io:rating-board";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_RATING: u8 = 5;
const MAX_ALLOWED_RATING: u8 = 100;
const MAX_NAME_LEN: usize = 80;
const MAX_DESCRIPTION_LEN: usize = 400;
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let max_rating = validate_max_rating(msg.max_rating)?;
    let config = Config {
        owner: info.sender.clone(),
        max_rating,
        next_item_id: 1,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("max_rating", max_rating.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateItem { name, description } => {
            execute_create_item(deps, info, name, description)
        }
        ExecuteMsg::RateItem { item_id, rating } => execute_rate_item(deps, info, item_id, rating),
        ExecuteMsg::UpdateConfig {
            max_rating,
            new_owner,
        } => execute_update_config(deps, info, max_rating, new_owner),
    }
}

pub fn execute_create_item(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    description: Option<String>,
) -> Result<Response, ContractError> {
    let name = validate_name(name)?;
    let description = validate_description(description)?;
    let mut config = CONFIG.load(deps.storage)?;
    let item_id = config.next_item_id;

    let item = RatedItem {
        creator: info.sender.clone(),
        name,
        description,
        total_rating: 0,
        rating_count: 0,
    };

    ITEMS.save(deps.storage, item_id, &item)?;
    config.next_item_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "create_item")
        .add_attribute("creator", info.sender)
        .add_attribute("item_id", item_id.to_string()))
}

pub fn execute_rate_item(
    deps: DepsMut,
    info: MessageInfo,
    item_id: u64,
    rating: u8,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    validate_rating(rating, config.max_rating)?;

    let mut item = ITEMS
        .may_load(deps.storage, item_id)?
        .ok_or(ContractError::ItemNotFound {})?;
    let previous_rating = RATINGS.may_load(deps.storage, (item_id, &info.sender))?;

    match previous_rating {
        Some(previous) => {
            item.total_rating = item.total_rating + rating as u64 - previous as u64;
        }
        None => {
            item.total_rating += rating as u64;
            item.rating_count += 1;
        }
    }

    ITEMS.save(deps.storage, item_id, &item)?;
    RATINGS.save(deps.storage, (item_id, &info.sender), &rating)?;

    Ok(Response::new()
        .add_attribute("action", "rate_item")
        .add_attribute("rater", info.sender)
        .add_attribute("item_id", item_id.to_string())
        .add_attribute("rating", rating.to_string())
        .add_attribute("rating_count", item.rating_count.to_string()))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    max_rating: Option<u8>,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if max_rating.is_some() {
        config.max_rating = validate_max_rating(max_rating)?;
    }
    if let Some(new_owner) = new_owner {
        config.owner = deps.api.addr_validate(&new_owner)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute("max_rating", config.max_rating.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Item { item_id } => to_json_binary(&query_item(deps, item_id)?),
        QueryMsg::Rating { item_id, rater } => to_json_binary(&query_rating(deps, item_id, rater)?),
        QueryMsg::Items { start_after, limit } => {
            to_json_binary(&query_items(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.into_string(),
        max_rating: config.max_rating,
        next_item_id: config.next_item_id,
    })
}

pub fn query_item(deps: Deps, item_id: u64) -> StdResult<ItemResponse> {
    let item = ITEMS.load(deps.storage, item_id)?;
    Ok(item_response(item_id, item))
}

pub fn query_rating(deps: Deps, item_id: u64, rater: String) -> StdResult<RatingResponse> {
    let rater_addr = deps.api.addr_validate(&rater)?;
    let rating = RATINGS.load(deps.storage, (item_id, &rater_addr))?;
    Ok(RatingResponse {
        item_id,
        rater: rater_addr.into_string(),
        rating,
    })
}

pub fn query_items(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ItemsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let items = ITEMS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (item_id, item) = item?;
            Ok(item_response(item_id, item))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ItemsResponse { items })
}

fn item_response(item_id: u64, item: RatedItem) -> ItemResponse {
    ItemResponse {
        item_id,
        creator: item.creator.into_string(),
        name: item.name,
        description: item.description,
        total_rating: item.total_rating,
        rating_count: item.rating_count,
        average_rating: average_rating(item.total_rating, item.rating_count),
    }
}

fn average_rating(total_rating: u64, rating_count: u64) -> Option<String> {
    if rating_count == 0 {
        return None;
    }
    let whole = total_rating / rating_count;
    let fractional = ((total_rating % rating_count) * 100) / rating_count;
    Some(format!("{whole}.{fractional:02}"))
}

fn validate_name(name: String) -> Result<String, ContractError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ContractError::EmptyName {});
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(ContractError::NameTooLong {});
    }
    Ok(name)
}

fn validate_description(description: Option<String>) -> Result<Option<String>, ContractError> {
    description
        .map(|value| {
            let value = value.trim().to_string();
            if value.chars().count() > MAX_DESCRIPTION_LEN {
                Err(ContractError::DescriptionTooLong {})
            } else if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        })
        .transpose()
        .map(Option::flatten)
}

fn validate_max_rating(max_rating: Option<u8>) -> Result<u8, ContractError> {
    let max_rating = max_rating.unwrap_or(DEFAULT_MAX_RATING);
    if max_rating == 0 || max_rating > MAX_ALLOWED_RATING {
        return Err(ContractError::InvalidMaxRating {});
    }
    Ok(max_rating)
}

fn validate_rating(rating: u8, max_rating: u8) -> Result<(), ContractError> {
    if rating == 0 || rating > max_rating {
        return Err(ContractError::InvalidRating {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{from_json, Env, MemoryStorage, OwnedDeps};

    fn setup() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            InstantiateMsg {
                max_rating: Some(5),
            },
        )
        .unwrap();
        (deps, env)
    }

    fn create_item(deps: DepsMut, env: Env, creator: &str) {
        execute(
            deps,
            env,
            mock_info(creator, &[]),
            ExecuteMsg::CreateItem {
                name: "Example".to_string(),
                description: Some("A rated item".to_string()),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_sets_config() {
        let (deps, env) = setup();
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.owner, "admin");
        assert_eq!(config.max_rating, 5);
        assert_eq!(config.next_item_id, 1);
    }

    #[test]
    fn users_can_create_items() {
        let (mut deps, env) = setup();
        create_item(deps.as_mut(), env.clone(), "creator");

        let res = query(deps.as_ref(), env, QueryMsg::Item { item_id: 1 }).unwrap();
        let item: ItemResponse = from_json(&res).unwrap();

        assert_eq!(item.item_id, 1);
        assert_eq!(item.creator, "creator");
        assert_eq!(item.name, "Example");
        assert_eq!(item.description, Some("A rated item".to_string()));
        assert_eq!(item.rating_count, 0);
        assert_eq!(item.average_rating, None);
    }

    #[test]
    fn rating_updates_aggregate_totals() {
        let (mut deps, env) = setup();
        create_item(deps.as_mut(), env.clone(), "creator");

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::RateItem {
                item_id: 1,
                rating: 4,
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("bob", &[]),
            ExecuteMsg::RateItem {
                item_id: 1,
                rating: 2,
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::RateItem {
                item_id: 1,
                rating: 5,
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env, QueryMsg::Item { item_id: 1 }).unwrap();
        let item: ItemResponse = from_json(&res).unwrap();

        assert_eq!(item.total_rating, 7);
        assert_eq!(item.rating_count, 2);
        assert_eq!(item.average_rating, Some("3.50".to_string()));
    }

    #[test]
    fn invalid_names_and_ratings_are_rejected() {
        let (mut deps, env) = setup();

        let empty_name = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::CreateItem {
                name: " ".to_string(),
                description: None,
            },
        )
        .unwrap_err();
        assert_eq!(empty_name, ContractError::EmptyName {});

        create_item(deps.as_mut(), env.clone(), "creator");
        let invalid_rating = execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::RateItem {
                item_id: 1,
                rating: 6,
            },
        )
        .unwrap_err();
        assert_eq!(invalid_rating, ContractError::InvalidRating {});
    }

    #[test]
    fn item_and_rating_queries_work() {
        let (mut deps, env) = setup();
        create_item(deps.as_mut(), env.clone(), "creator");
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::RateItem {
                item_id: 1,
                rating: 5,
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Rating {
                item_id: 1,
                rater: "alice".to_string(),
            },
        )
        .unwrap();
        let rating: RatingResponse = from_json(&res).unwrap();

        assert_eq!(rating.item_id, 1);
        assert_eq!(rating.rater, "alice");
        assert_eq!(rating.rating, 5);
    }

    #[test]
    fn items_query_is_paginated() {
        let (mut deps, env) = setup();
        for creator in ["alice", "bob", "carol"] {
            create_item(deps.as_mut(), env.clone(), creator);
        }

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Items {
                start_after: Some(1),
                limit: Some(2),
            },
        )
        .unwrap();
        let items: ItemsResponse = from_json(&res).unwrap();

        assert_eq!(items.items.len(), 2);
        assert_eq!(items.items[0].item_id, 2);
        assert_eq!(items.items[1].item_id, 3);
    }

    #[test]
    fn only_owner_can_update_config() {
        let (mut deps, env) = setup();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateConfig {
                max_rating: Some(10),
                new_owner: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                max_rating: Some(10),
                new_owner: Some("alice".to_string()),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.owner, "alice");
        assert_eq!(config.max_rating, 10);
    }
}
