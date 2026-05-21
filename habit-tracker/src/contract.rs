#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RecordResponse, RecordsResponse,
};
use crate::state::{Config, HabitRecord, CONFIG, RECORDS};

const CONTRACT_NAME: &str = "crates.io:habit-tracker";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MIN_GAP: u64 = 1;
const DEFAULT_MAX_GAP: u64 = 100;
const DEFAULT_MAX_NOTE_LEN: u16 = 160;
const MAX_ALLOWED_NOTE_LEN: u16 = 500;
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = validate_config(
        info.sender.clone(),
        msg.min_gap,
        msg.max_gap,
        msg.max_note_len,
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("min_gap", config.min_gap.to_string())
        .add_attribute("max_gap", config.max_gap.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CheckIn { note } => execute_check_in(deps, env, info, note),
        ExecuteMsg::Reset {} => execute_reset(deps, info),
        ExecuteMsg::UpdateConfig {
            min_gap,
            max_gap,
            max_note_len,
            new_owner,
        } => execute_update_config(deps, info, min_gap, max_gap, max_note_len, new_owner),
    }
}

pub fn execute_check_in(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    note: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let note = validate_note(note, config.max_note_len)?;
    let height = env.block.height;

    let previous = RECORDS.may_load(deps.storage, &info.sender)?;
    let record = match previous {
        Some(record) => advance_record(record, height, &config)?,
        None => HabitRecord {
            last_check_in_height: height,
            current_streak: 1,
            best_streak: 1,
            total_check_ins: 1,
            note: None,
        },
    };

    let mut record = record;
    record.note = note;
    RECORDS.save(deps.storage, &info.sender, &record)?;

    Ok(Response::new()
        .add_attribute("action", "check_in")
        .add_attribute("user", info.sender)
        .add_attribute("height", height.to_string())
        .add_attribute("current_streak", record.current_streak.to_string())
        .add_attribute("best_streak", record.best_streak.to_string())
        .add_attribute("total_check_ins", record.total_check_ins.to_string()))
}

pub fn execute_reset(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    RECORDS.remove(deps.storage, &info.sender);

    Ok(Response::new()
        .add_attribute("action", "reset")
        .add_attribute("user", info.sender))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    min_gap: Option<u64>,
    max_gap: Option<u64>,
    max_note_len: Option<u16>,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let owner = match new_owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => config.owner,
    };
    let updated = validate_config(
        owner,
        min_gap.or(Some(config.min_gap)),
        max_gap.or(Some(config.max_gap)),
        max_note_len.or(Some(config.max_note_len)),
    )?;
    CONFIG.save(deps.storage, &updated)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", updated.owner)
        .add_attribute("min_gap", updated.min_gap.to_string())
        .add_attribute("max_gap", updated.max_gap.to_string())
        .add_attribute("max_note_len", updated.max_note_len.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Record { user } => to_json_binary(&query_record(deps, user)?),
        QueryMsg::Records { start_after, limit } => {
            to_json_binary(&query_records(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.into_string(),
        min_gap: config.min_gap,
        max_gap: config.max_gap,
        max_note_len: config.max_note_len,
    })
}

pub fn query_record(deps: Deps, user: String) -> StdResult<RecordResponse> {
    let user = deps.api.addr_validate(&user)?;
    let record = RECORDS.load(deps.storage, &user)?;
    Ok(record_response(user, record))
}

pub fn query_records(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<RecordsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start_after = start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?;
    let start = start_after.as_ref().map(Bound::exclusive);

    let records = RECORDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|record| {
            let (user, record) = record?;
            Ok(record_response(user, record))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(RecordsResponse { records })
}

fn advance_record(
    record: HabitRecord,
    height: u64,
    config: &Config,
) -> Result<HabitRecord, ContractError> {
    let next_allowed_height = record.last_check_in_height.saturating_add(config.min_gap);
    if height < next_allowed_height {
        return Err(ContractError::TooSoon {
            next_allowed_height,
        });
    }

    let current_streak = if height <= record.last_check_in_height.saturating_add(config.max_gap) {
        record.current_streak + 1
    } else {
        1
    };
    let best_streak = record.best_streak.max(current_streak);

    Ok(HabitRecord {
        last_check_in_height: height,
        current_streak,
        best_streak,
        total_check_ins: record.total_check_ins + 1,
        note: record.note,
    })
}

fn record_response(user: Addr, record: HabitRecord) -> RecordResponse {
    RecordResponse {
        user: user.into_string(),
        last_check_in_height: record.last_check_in_height,
        current_streak: record.current_streak,
        best_streak: record.best_streak,
        total_check_ins: record.total_check_ins,
        note: record.note,
    }
}

fn validate_config(
    owner: Addr,
    min_gap: Option<u64>,
    max_gap: Option<u64>,
    max_note_len: Option<u16>,
) -> Result<Config, ContractError> {
    let min_gap = min_gap.unwrap_or(DEFAULT_MIN_GAP);
    if min_gap == 0 {
        return Err(ContractError::InvalidMinGap {});
    }

    let max_gap = max_gap.unwrap_or(DEFAULT_MAX_GAP);
    if max_gap < min_gap {
        return Err(ContractError::InvalidMaxGap {});
    }

    let max_note_len = max_note_len.unwrap_or(DEFAULT_MAX_NOTE_LEN);
    if max_note_len == 0 || max_note_len > MAX_ALLOWED_NOTE_LEN {
        return Err(ContractError::InvalidMaxNoteLen {});
    }

    Ok(Config {
        owner,
        min_gap,
        max_gap,
        max_note_len,
    })
}

fn validate_note(note: Option<String>, max_note_len: u16) -> Result<Option<String>, ContractError> {
    note.map(|note| {
        let note = note.trim().to_string();
        if note.chars().count() > max_note_len as usize {
            Err(ContractError::NoteTooLong {})
        } else if note.is_empty() {
            Ok(None)
        } else {
            Ok(Some(note))
        }
    })
    .transpose()
    .map(Option::flatten)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{from_json, Env, MemoryStorage, OwnedDeps};

    fn setup() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.height = 100;
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            InstantiateMsg {
                min_gap: Some(10),
                max_gap: Some(30),
                max_note_len: Some(20),
            },
        )
        .unwrap();
        (deps, env)
    }

    fn check_in(deps: DepsMut, mut env: Env, user: &str, height: u64, note: Option<&str>) -> Env {
        env.block.height = height;
        execute(
            deps,
            env.clone(),
            mock_info(user, &[]),
            ExecuteMsg::CheckIn {
                note: note.map(str::to_string),
            },
        )
        .unwrap();
        env
    }

    #[test]
    fn instantiate_sets_config() {
        let (deps, env) = setup();
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.owner, "admin");
        assert_eq!(config.min_gap, 10);
        assert_eq!(config.max_gap, 30);
        assert_eq!(config.max_note_len, 20);
    }

    #[test]
    fn first_check_in_creates_record() {
        let (mut deps, env) = setup();
        let env = check_in(deps.as_mut(), env, "alice", 100, Some("  learned maps  "));

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Record {
                user: "alice".to_string(),
            },
        )
        .unwrap();
        let record: RecordResponse = from_json(&res).unwrap();

        assert_eq!(record.user, "alice");
        assert_eq!(record.last_check_in_height, 100);
        assert_eq!(record.current_streak, 1);
        assert_eq!(record.best_streak, 1);
        assert_eq!(record.total_check_ins, 1);
        assert_eq!(record.note, Some("learned maps".to_string()));
    }

    #[test]
    fn check_in_enforces_minimum_gap() {
        let (mut deps, env) = setup();
        let mut env = check_in(deps.as_mut(), env, "alice", 100, None);
        env.block.height = 109;

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::CheckIn { note: None },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::TooSoon {
                next_allowed_height: 110
            }
        );
    }

    #[test]
    fn check_ins_extend_or_reset_streaks() {
        let (mut deps, env) = setup();
        let env = check_in(deps.as_mut(), env, "alice", 100, None);
        let env = check_in(deps.as_mut(), env, "alice", 110, None);
        let env = check_in(deps.as_mut(), env, "alice", 200, Some("back again"));

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Record {
                user: "alice".to_string(),
            },
        )
        .unwrap();
        let record: RecordResponse = from_json(&res).unwrap();

        assert_eq!(record.last_check_in_height, 200);
        assert_eq!(record.current_streak, 1);
        assert_eq!(record.best_streak, 2);
        assert_eq!(record.total_check_ins, 3);
        assert_eq!(record.note, Some("back again".to_string()));
    }

    #[test]
    fn reset_removes_sender_record() {
        let (mut deps, env) = setup();
        let env = check_in(deps.as_mut(), env, "alice", 100, None);

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::Reset {},
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Records {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let records: RecordsResponse = from_json(&res).unwrap();

        assert!(records.records.is_empty());
    }

    #[test]
    fn only_owner_updates_config() {
        let (mut deps, env) = setup();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateConfig {
                min_gap: Some(1),
                max_gap: None,
                max_note_len: None,
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
                min_gap: Some(5),
                max_gap: Some(50),
                max_note_len: Some(30),
                new_owner: Some("new_admin".to_string()),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.owner, "new_admin");
        assert_eq!(config.min_gap, 5);
        assert_eq!(config.max_gap, 50);
        assert_eq!(config.max_note_len, 30);
    }

    #[test]
    fn records_query_is_paginated_by_address() {
        let (mut deps, env) = setup();
        let env = check_in(deps.as_mut(), env, "alice", 100, None);
        let env = check_in(deps.as_mut(), env, "bob", 100, None);
        let env = check_in(deps.as_mut(), env, "carol", 100, None);

        let first_page = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Records {
                start_after: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let first_page: RecordsResponse = from_json(&first_page).unwrap();

        assert_eq!(first_page.records.len(), 2);
        assert_eq!(first_page.records[0].user, "alice");
        assert_eq!(first_page.records[1].user, "bob");

        let second_page = query(
            deps.as_ref(),
            env,
            QueryMsg::Records {
                start_after: Some("bob".to_string()),
                limit: Some(2),
            },
        )
        .unwrap();
        let second_page: RecordsResponse = from_json(&second_page).unwrap();

        assert_eq!(second_page.records.len(), 1);
        assert_eq!(second_page.records[0].user, "carol");
    }

    #[test]
    fn invalid_config_and_notes_are_rejected() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let invalid_gap = instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            InstantiateMsg {
                min_gap: Some(10),
                max_gap: Some(5),
                max_note_len: None,
            },
        )
        .unwrap_err();
        assert_eq!(invalid_gap, ContractError::InvalidMaxGap {});

        let (mut deps, env) = setup();
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::CheckIn {
                note: Some("this note is much too long for the configured limit".to_string()),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NoteTooLong {});
    }
}
