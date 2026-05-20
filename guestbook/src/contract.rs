#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, EntriesResponse, EntryResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use crate::state::{Config, Entry, CONFIG, ENTRIES};

const CONTRACT_NAME: &str = "crates.io:guestbook";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_MESSAGE_LEN: u32 = 280;
const MAX_ALLOWED_MESSAGE_LEN: u32 = 1000;
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let max_message_len = validate_max_message_len(msg.max_message_len)?;
    let config = Config {
        owner: info.sender.clone(),
        title: msg.title.trim().to_string(),
        max_message_len,
        entry_count: 0,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("title", config.title)
        .add_attribute("max_message_len", max_message_len.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Sign {
            display_name,
            message,
        } => execute_sign(deps, env, info, display_name, message),
        ExecuteMsg::RemoveEntry { signer } => execute_remove_entry(deps, info, signer),
        ExecuteMsg::UpdateConfig {
            title,
            max_message_len,
            new_owner,
        } => execute_update_config(deps, info, title, max_message_len, new_owner),
    }
}

pub fn execute_sign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    display_name: String,
    message: String,
) -> Result<Response, ContractError> {
    let display_name = display_name.trim().to_string();
    let message = message.trim().to_string();

    if display_name.is_empty() {
        return Err(ContractError::EmptyDisplayName {});
    }
    if message.is_empty() {
        return Err(ContractError::EmptyMessage {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    if message.chars().count() > config.max_message_len as usize {
        return Err(ContractError::MessageTooLong {});
    }

    let already_signed = ENTRIES.has(deps.storage, &info.sender);
    let entry = Entry {
        display_name,
        message,
        signed_at_height: env.block.height,
    };
    ENTRIES.save(deps.storage, &info.sender, &entry)?;

    if !already_signed {
        config.entry_count += 1;
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::new()
        .add_attribute("action", "sign")
        .add_attribute("signer", info.sender)
        .add_attribute("entry_count", config.entry_count.to_string()))
}

pub fn execute_remove_entry(
    deps: DepsMut,
    info: MessageInfo,
    signer: String,
) -> Result<Response, ContractError> {
    let signer_addr = deps.api.addr_validate(&signer)?;
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != signer_addr && info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if ENTRIES.has(deps.storage, &signer_addr) {
        ENTRIES.remove(deps.storage, &signer_addr);
        config.entry_count -= 1;
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::new()
        .add_attribute("action", "remove_entry")
        .add_attribute("signer", signer_addr)
        .add_attribute("entry_count", config.entry_count.to_string()))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    title: Option<String>,
    max_message_len: Option<u32>,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(title) = title {
        config.title = title.trim().to_string();
    }
    if max_message_len.is_some() {
        config.max_message_len = validate_max_message_len(max_message_len)?;
    }
    if let Some(new_owner) = new_owner {
        config.owner = deps.api.addr_validate(&new_owner)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute("max_message_len", config.max_message_len.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Entry { signer } => to_json_binary(&query_entry(deps, signer)?),
        QueryMsg::Entries { start_after, limit } => {
            to_json_binary(&query_entries(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.into_string(),
        title: config.title,
        max_message_len: config.max_message_len,
        entry_count: config.entry_count,
    })
}

pub fn query_entry(deps: Deps, signer: String) -> StdResult<EntryResponse> {
    let signer_addr = deps.api.addr_validate(&signer)?;
    let entry = ENTRIES.load(deps.storage, &signer_addr)?;
    Ok(entry_response(signer_addr, entry))
}

pub fn query_entries(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<EntriesResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start_addr = start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;
    let start = start_addr.as_ref().map(Bound::exclusive);

    let entries = ENTRIES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (signer, entry) = item?;
            Ok(entry_response(signer, entry))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(EntriesResponse { entries })
}

fn entry_response(signer: cosmwasm_std::Addr, entry: Entry) -> EntryResponse {
    EntryResponse {
        signer: signer.into_string(),
        display_name: entry.display_name,
        message: entry.message,
        signed_at_height: entry.signed_at_height,
    }
}

fn validate_max_message_len(max_message_len: Option<u32>) -> Result<u32, ContractError> {
    let max_message_len = max_message_len.unwrap_or(DEFAULT_MAX_MESSAGE_LEN);
    if max_message_len == 0 || max_message_len > MAX_ALLOWED_MESSAGE_LEN {
        return Err(ContractError::InvalidMaxMessageLen {});
    }
    Ok(max_message_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{from_json, Env, MemoryStorage, OwnedDeps};

    fn setup() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {
            title: "Launch guestbook".to_string(),
            max_message_len: Some(40),
        };
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        (deps, env)
    }

    #[test]
    fn instantiate_sets_config() {
        let (deps, env) = setup();
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.owner, "owner");
        assert_eq!(config.title, "Launch guestbook");
        assert_eq!(config.max_message_len, 40);
        assert_eq!(config.entry_count, 0);
    }

    #[test]
    fn signer_can_add_and_update_entry() {
        let (mut deps, mut env) = setup();
        env.block.height = 123;

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("signer", &[]),
            ExecuteMsg::Sign {
                display_name: " Ada ".to_string(),
                message: " First note ".to_string(),
            },
        )
        .unwrap();

        env.block.height = 124;
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("signer", &[]),
            ExecuteMsg::Sign {
                display_name: "Ada".to_string(),
                message: "Updated note".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Entry {
                signer: "signer".to_string(),
            },
        )
        .unwrap();
        let entry: EntryResponse = from_json(&res).unwrap();

        assert_eq!(entry.signer, "signer");
        assert_eq!(entry.display_name, "Ada");
        assert_eq!(entry.message, "Updated note");
        assert_eq!(entry.signed_at_height, 124);

        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.entry_count, 1);
    }

    #[test]
    fn message_validation_rejects_empty_or_long_values() {
        let (mut deps, env) = setup();

        let empty_name = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("signer", &[]),
            ExecuteMsg::Sign {
                display_name: " ".to_string(),
                message: "hello".to_string(),
            },
        );
        assert_eq!(empty_name.unwrap_err(), ContractError::EmptyDisplayName {});

        let long_message = execute(
            deps.as_mut(),
            env,
            mock_info("signer", &[]),
            ExecuteMsg::Sign {
                display_name: "Ada".to_string(),
                message: "x".repeat(41),
            },
        );
        assert_eq!(long_message.unwrap_err(), ContractError::MessageTooLong {});
    }

    #[test]
    fn only_owner_or_signer_can_remove_entry() {
        let (mut deps, env) = setup();
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("signer", &[]),
            ExecuteMsg::Sign {
                display_name: "Ada".to_string(),
                message: "hello".to_string(),
            },
        )
        .unwrap();

        let unauthorized = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("stranger", &[]),
            ExecuteMsg::RemoveEntry {
                signer: "signer".to_string(),
            },
        );
        assert_eq!(unauthorized.unwrap_err(), ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::RemoveEntry {
                signer: "signer".to_string(),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.entry_count, 0);
    }

    #[test]
    fn entries_query_is_paginated() {
        let (mut deps, env) = setup();

        for signer in ["addr0001", "addr0002", "addr0003"] {
            execute(
                deps.as_mut(),
                env.clone(),
                mock_info(signer, &[]),
                ExecuteMsg::Sign {
                    display_name: signer.to_string(),
                    message: "hello".to_string(),
                },
            )
            .unwrap();
        }

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Entries {
                start_after: Some("addr0001".to_string()),
                limit: Some(2),
            },
        )
        .unwrap();
        let entries: EntriesResponse = from_json(&res).unwrap();

        assert_eq!(entries.entries.len(), 2);
        assert_eq!(entries.entries[0].signer, "addr0002");
        assert_eq!(entries.entries[1].signer, "addr0003");
    }

    #[test]
    fn owner_can_update_config_and_transfer_ownership() {
        let (mut deps, env) = setup();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                title: Some("Updated guestbook".to_string()),
                max_message_len: Some(100),
                new_owner: Some("new_owner".to_string()),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.owner, "new_owner");
        assert_eq!(config.title, "Updated guestbook");
        assert_eq!(config.max_message_len, 100);
    }
}
