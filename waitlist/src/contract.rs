#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, EntryResponse, EntryView, ExecuteMsg, InstantiateMsg, ListEntriesResponse,
    QueryMsg, StatsResponse,
};
use crate::state::{
    Config, Entry, EntryStatus, ADMITTED_COUNT, CONFIG, ENTRIES, NEXT_POSITION, WAITING_COUNT,
};

const CONTRACT_NAME: &str = "crates.io:waitlist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_NOTE_LENGTH: u32 = 120;
const DEFAULT_MAX_ENTRIES: u32 = 500;
const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        max_note_length: msg.max_note_length.unwrap_or(DEFAULT_MAX_NOTE_LENGTH),
        max_entries: msg.max_entries.unwrap_or(DEFAULT_MAX_ENTRIES),
    };
    validate_limits(config.max_note_length, config.max_entries)?;

    CONFIG.save(deps.storage, &config)?;
    NEXT_POSITION.save(deps.storage, &1)?;
    WAITING_COUNT.save(deps.storage, &0)?;
    ADMITTED_COUNT.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Join { note } => join(deps, env, info, note),
        ExecuteMsg::UpdateNote { note } => update_note(deps, info, note),
        ExecuteMsg::Leave {} => leave(deps, info),
        ExecuteMsg::Admit { member } => admit(deps, info, member),
        ExecuteMsg::Remove { member } => remove(deps, info, member),
        ExecuteMsg::UpdateConfig {
            owner,
            max_note_length,
            max_entries,
        } => update_config(deps, info, owner, max_note_length, max_entries),
    }
}

pub fn join(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    note: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let active_count = active_count(deps.storage)?;
    if active_count >= config.max_entries {
        return Err(ContractError::WaitlistFull {});
    }
    if ENTRIES.has(deps.storage, &info.sender) {
        return Err(ContractError::AlreadyJoined {});
    }

    let note = validate_note(note, config.max_note_length)?;
    let position = NEXT_POSITION.load(deps.storage)?;
    let entry = Entry {
        member: info.sender.clone(),
        note,
        position,
        joined_at_height: env.block.height,
        status: EntryStatus::Waiting,
    };

    ENTRIES.save(deps.storage, &info.sender, &entry)?;
    NEXT_POSITION.save(deps.storage, &(position + 1))?;
    increment_waiting(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "join")
        .add_attribute("member", info.sender)
        .add_attribute("position", position.to_string()))
}

pub fn update_note(
    deps: DepsMut,
    info: MessageInfo,
    note: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let note = validate_note(note, config.max_note_length)?;

    ENTRIES.update(
        deps.storage,
        &info.sender,
        |entry| -> Result<Entry, ContractError> {
            let mut entry = entry.ok_or(ContractError::EntryNotFound {})?;
            entry.note = note;
            Ok(entry)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "update_note")
        .add_attribute("member", info.sender))
}

pub fn leave(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let entry = ENTRIES
        .may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::EntryNotFound {})?;

    decrement_status_count(deps.storage, &entry.status)?;
    ENTRIES.remove(deps.storage, &info.sender);

    Ok(Response::new()
        .add_attribute("action", "leave")
        .add_attribute("member", info.sender))
}

pub fn admit(deps: DepsMut, info: MessageInfo, member: String) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;
    let member = deps.api.addr_validate(&member)?;

    ENTRIES.update(
        deps.storage,
        &member,
        |entry| -> Result<Entry, ContractError> {
            let mut entry = entry.ok_or(ContractError::EntryNotFound {})?;
            if entry.status == EntryStatus::Admitted {
                return Err(ContractError::AlreadyAdmitted {});
            }
            entry.status = EntryStatus::Admitted;
            Ok(entry)
        },
    )?;
    decrement_waiting(deps.storage)?;
    increment_admitted(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "admit")
        .add_attribute("member", member))
}

pub fn remove(deps: DepsMut, info: MessageInfo, member: String) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;
    let member = deps.api.addr_validate(&member)?;
    let entry = ENTRIES
        .may_load(deps.storage, &member)?
        .ok_or(ContractError::EntryNotFound {})?;

    decrement_status_count(deps.storage, &entry.status)?;
    ENTRIES.remove(deps.storage, &member);

    Ok(Response::new()
        .add_attribute("action", "remove")
        .add_attribute("member", member))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    max_note_length: Option<u32>,
    max_entries: Option<u32>,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;

    let current_active = active_count(deps.storage)?;
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(max_note_length) = max_note_length {
            config.max_note_length = max_note_length;
        }
        if let Some(max_entries) = max_entries {
            config.max_entries = max_entries;
        }
        validate_limits(config.max_note_length, config.max_entries)?;
        if config.max_entries < current_active {
            return Err(ContractError::LimitBelowActiveEntries {});
        }
        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Entry { member } => to_json_binary(&query_entry(deps, member)?),
        QueryMsg::Stats {} => to_json_binary(&query_stats(deps)?),
        QueryMsg::ListEntries { start_after, limit } => {
            to_json_binary(&query_list_entries(deps, start_after, limit)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _msg: crate::msg::MigrateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "migrate"))
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        max_note_length: config.max_note_length,
        max_entries: config.max_entries,
    })
}

fn query_entry(deps: Deps, member: String) -> StdResult<EntryResponse> {
    let member = deps.api.addr_validate(&member)?;
    let entry = ENTRIES.may_load(deps.storage, &member)?;
    Ok(EntryResponse {
        entry: entry.map(entry_to_response),
    })
}

fn query_stats(deps: Deps) -> StdResult<StatsResponse> {
    let waiting = WAITING_COUNT.load(deps.storage)?;
    let admitted = ADMITTED_COUNT.load(deps.storage)?;
    Ok(StatsResponse {
        waiting,
        admitted,
        active: waiting + admitted,
        next_position: NEXT_POSITION.load(deps.storage)?,
    })
}

fn query_list_entries(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ListEntriesResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_after = start_after
        .map(|member| deps.api.addr_validate(&member))
        .transpose()?;
    let start = start_after.as_ref().map(Bound::exclusive);

    let entries = ENTRIES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|entry| entry.map(|(_, entry)| entry_to_response(entry)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ListEntriesResponse { entries })
}

fn assert_owner(deps: Deps, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != *sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn entry_to_response(entry: Entry) -> EntryView {
    EntryView {
        member: entry.member.to_string(),
        note: entry.note,
        position: entry.position,
        joined_at_height: entry.joined_at_height,
        status: entry.status,
    }
}

fn validate_limits(max_note_length: u32, max_entries: u32) -> Result<(), ContractError> {
    if max_note_length == 0 || max_entries == 0 {
        return Err(ContractError::InvalidLimit {});
    }
    Ok(())
}

fn validate_note(
    note: Option<String>,
    max_note_length: u32,
) -> Result<Option<String>, ContractError> {
    note.map(|note| {
        let note = note.trim().to_string();
        if note.is_empty() {
            return Err(ContractError::EmptyNote {});
        }
        if note.len() > max_note_length as usize {
            return Err(ContractError::NoteTooLong {});
        }
        Ok(note)
    })
    .transpose()
}

fn active_count(storage: &dyn cosmwasm_std::Storage) -> StdResult<u32> {
    Ok(WAITING_COUNT.load(storage)? + ADMITTED_COUNT.load(storage)?)
}

fn increment_waiting(storage: &mut dyn cosmwasm_std::Storage) -> StdResult<()> {
    WAITING_COUNT.update(storage, |count| -> StdResult<_> { Ok(count + 1) })?;
    Ok(())
}

fn decrement_waiting(storage: &mut dyn cosmwasm_std::Storage) -> StdResult<()> {
    WAITING_COUNT.update(storage, |count| -> StdResult<_> {
        Ok(count.saturating_sub(1))
    })?;
    Ok(())
}

fn increment_admitted(storage: &mut dyn cosmwasm_std::Storage) -> StdResult<()> {
    ADMITTED_COUNT.update(storage, |count| -> StdResult<_> { Ok(count + 1) })?;
    Ok(())
}

fn decrement_admitted(storage: &mut dyn cosmwasm_std::Storage) -> StdResult<()> {
    ADMITTED_COUNT.update(storage, |count| -> StdResult<_> {
        Ok(count.saturating_sub(1))
    })?;
    Ok(())
}

fn decrement_status_count(
    storage: &mut dyn cosmwasm_std::Storage,
    status: &EntryStatus,
) -> StdResult<()> {
    match status {
        EntryStatus::Waiting => decrement_waiting(storage),
        EntryStatus::Admitted => decrement_admitted(storage),
    }
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
                max_note_length: Some(20),
                max_entries: Some(2),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_sets_owner_and_defaults() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                max_note_length: None,
                max_entries: None,
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.owner, "creator");
        assert_eq!(config.max_note_length, DEFAULT_MAX_NOTE_LENGTH);
        assert_eq!(config.max_entries, DEFAULT_MAX_ENTRIES);
    }

    #[test]
    fn join_records_position_and_stats() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join {
                note: Some("beta".to_string()),
            },
        )
        .unwrap();

        let entry: EntryResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Entry {
                    member: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let entry = entry.entry.unwrap();
        assert_eq!(entry.member, "alice");
        assert_eq!(entry.note, Some("beta".to_string()));
        assert_eq!(entry.position, 1);
        assert_eq!(entry.status, EntryStatus::Waiting);

        let stats: StatsResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Stats {}).unwrap()).unwrap();
        assert_eq!(stats.waiting, 1);
        assert_eq!(stats.admitted, 0);
        assert_eq!(stats.active, 1);
        assert_eq!(stats.next_position, 2);
    }

    #[test]
    fn joining_twice_is_rejected() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join { note: None },
        )
        .unwrap();
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join { note: None },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::AlreadyJoined {});
    }

    #[test]
    fn note_validation_rejects_empty_or_long_notes() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let empty = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join {
                note: Some("   ".to_string()),
            },
        )
        .unwrap_err();
        assert_eq!(empty, ContractError::EmptyNote {});

        let long = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join {
                note: Some("this note is definitely too long".to_string()),
            },
        )
        .unwrap_err();
        assert_eq!(long, ContractError::NoteTooLong {});
    }

    #[test]
    fn waitlist_capacity_is_enforced() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        for member in ["alice", "bob"] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(member, &[]),
                ExecuteMsg::Join { note: None },
            )
            .unwrap();
        }

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("carol", &[]),
            ExecuteMsg::Join { note: None },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::WaitlistFull {});
    }

    #[test]
    fn owner_can_admit_and_remove_members() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join { note: None },
        )
        .unwrap();

        let unauthorized = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Admit {
                member: "alice".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(unauthorized, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Admit {
                member: "alice".to_string(),
            },
        )
        .unwrap();

        let stats: StatsResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Stats {}).unwrap()).unwrap();
        assert_eq!(stats.waiting, 0);
        assert_eq!(stats.admitted, 1);

        let already = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Admit {
                member: "alice".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(already, ContractError::AlreadyAdmitted {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::Remove {
                member: "alice".to_string(),
            },
        )
        .unwrap();

        let stats: StatsResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Stats {}).unwrap()).unwrap();
        assert_eq!(stats.active, 0);
    }

    #[test]
    fn members_can_update_notes_and_leave() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join { note: None },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateNote {
                note: Some("ready".to_string()),
            },
        )
        .unwrap();

        let entry: EntryResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Entry {
                    member: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(entry.entry.unwrap().note, Some("ready".to_string()));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Leave {},
        )
        .unwrap();
        let stats: StatsResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Stats {}).unwrap()).unwrap();
        assert_eq!(stats.active, 0);
    }

    #[test]
    fn update_config_transfers_owner_and_rejects_too_small_capacity() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Join { note: None },
        )
        .unwrap();

        let too_small = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                owner: None,
                max_note_length: None,
                max_entries: Some(0),
            },
        )
        .unwrap_err();
        assert_eq!(too_small, ContractError::InvalidLimit {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                owner: Some("new_admin".to_string()),
                max_note_length: Some(30),
                max_entries: Some(3),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.owner, "new_admin");
        assert_eq!(config.max_note_length, 30);
        assert_eq!(config.max_entries, 3);
    }

    #[test]
    fn list_entries_supports_pagination() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        for member in ["alice", "bob"] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(member, &[]),
                ExecuteMsg::Join { note: None },
            )
            .unwrap();
        }

        let first_page: ListEntriesResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListEntries {
                    start_after: None,
                    limit: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(first_page.entries.len(), 1);
        assert_eq!(first_page.entries[0].member, "alice");

        let second_page: ListEntriesResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListEntries {
                    start_after: Some("alice".to_string()),
                    limit: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(second_page.entries.len(), 1);
        assert_eq!(second_page.entries[0].member, "bob");
    }
}
