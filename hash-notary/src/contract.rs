#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, ListRecordsResponse, QueryMsg, RecordResponse, StateResponse,
};
use crate::state::{Config, Record, CONFIG, DIGEST_INDEX, RECORDS, RECORD_SEQ};

const CONTRACT_NAME: &str = "crates.io:hash-notary";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DIGEST_HEX_LEN: usize = 64;
const LABEL_MAX_LEN: usize = 80;
const MEMO_MAX_LEN: usize = 240;
const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender.clone(),
            total_records: 0,
        },
    )?;
    RECORD_SEQ.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
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
        ExecuteMsg::Register {
            digest,
            label,
            memo,
        } => execute_register(deps, env, info, digest, label, memo),
        ExecuteMsg::UpdateMemo { id, memo } => execute_update_memo(deps, info, id, memo),
        ExecuteMsg::Revoke { id } => execute_revoke(deps, info, id),
    }
}

pub fn execute_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    digest: String,
    label: String,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    let digest = normalize_digest(&digest)?;
    if DIGEST_INDEX.may_load(deps.storage, &digest)?.is_some() {
        return Err(ContractError::DuplicateDigest {});
    }

    let label = normalize_label(&label)?;
    let memo = normalize_memo(memo)?;
    let id = RECORD_SEQ.update(deps.storage, |id| -> StdResult<_> { Ok(id + 1) })?;

    let record = Record {
        id,
        digest: digest.clone(),
        owner: info.sender.clone(),
        created_at: env.block.time.seconds(),
        label,
        memo,
        active: true,
    };

    RECORDS.save(deps.storage, id, &record)?;
    DIGEST_INDEX.save(deps.storage, &digest, &id)?;
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.total_records += 1;
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "register")
        .add_attribute("id", id.to_string())
        .add_attribute("digest", digest)
        .add_attribute("owner", info.sender))
}

pub fn execute_update_memo(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    memo: Option<String>,
) -> Result<Response, ContractError> {
    let memo = normalize_memo(memo)?;
    RECORDS.update(deps.storage, id, |record| -> Result<_, ContractError> {
        let mut record = record.ok_or(ContractError::RecordNotFound {})?;
        if record.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }
        record.memo = memo;
        Ok(record)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_memo")
        .add_attribute("id", id.to_string()))
}

pub fn execute_revoke(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    RECORDS.update(deps.storage, id, |record| -> Result<_, ContractError> {
        let mut record = record.ok_or(ContractError::RecordNotFound {})?;
        if record.owner != info.sender {
            return Err(ContractError::Unauthorized {});
        }
        record.active = false;
        Ok(record)
    })?;

    Ok(Response::new()
        .add_attribute("action", "revoke")
        .add_attribute("id", id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_json_binary(&query_state(deps)?),
        QueryMsg::GetRecord { id } => to_json_binary(&query_record(deps, id)?),
        QueryMsg::GetRecordByDigest { digest } => {
            to_json_binary(&query_record_by_digest(deps, digest)?)
        }
        QueryMsg::ListRecords { start_after, limit } => {
            to_json_binary(&query_records(deps, start_after, limit)?)
        }
    }
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(StateResponse {
        owner: config.owner,
        total_records: config.total_records,
    })
}

pub fn query_record(deps: Deps, id: u64) -> StdResult<RecordResponse> {
    let record = RECORDS.load(deps.storage, id)?;
    Ok(record.into())
}

pub fn query_record_by_digest(deps: Deps, digest: String) -> StdResult<RecordResponse> {
    let digest = normalize_digest(&digest)
        .map_err(|err| cosmwasm_std::StdError::generic_err(err.to_string()))?;
    let id = DIGEST_INDEX.load(deps.storage, &digest)?;
    query_record(deps, id)
}

pub fn query_records(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ListRecordsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let records = RECORDS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, record)| record.into()))
        .collect::<StdResult<Vec<RecordResponse>>>()?;

    Ok(ListRecordsResponse { records })
}

fn normalize_digest(digest: &str) -> Result<String, ContractError> {
    let trimmed = digest.trim();
    let without_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let normalized = without_prefix.to_ascii_lowercase();

    if normalized.len() != DIGEST_HEX_LEN
        || !normalized.chars().all(|char| char.is_ascii_hexdigit())
    {
        return Err(ContractError::InvalidDigest {});
    }

    Ok(normalized)
}

fn normalize_label(label: &str) -> Result<String, ContractError> {
    let label = label.trim();
    if label.is_empty() || label.len() > LABEL_MAX_LEN {
        return Err(ContractError::InvalidLabel {});
    }
    Ok(label.to_string())
}

fn normalize_memo(memo: Option<String>) -> Result<Option<String>, ContractError> {
    match memo {
        None => Ok(None),
        Some(memo) => {
            let memo = memo.trim();
            if memo.is_empty() {
                return Ok(None);
            }
            if memo.len() > MEMO_MAX_LEN {
                return Err(ContractError::InvalidMemo {});
            }
            Ok(Some(memo.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_json, Attribute};

    const DIGEST_A: &str = "b94d27b9934d3e08a52e52d7da7dabfade035a3e5094b9f9f1795f7c4f3819d0";
    const DIGEST_B: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    fn init() -> cosmwasm_std::OwnedDeps<
        cosmwasm_std::MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    > {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();
        deps
    }

    #[test]
    fn registers_and_queries_record() {
        let mut deps = init();
        let info = mock_info("alice", &[]);
        let msg = ExecuteMsg::Register {
            digest: format!("0x{}", DIGEST_A.to_uppercase()),
            label: " service-agreement.pdf ".to_string(),
            memo: Some(" Signed PDF hash ".to_string()),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "register"),
                Attribute::new("id", "1"),
                Attribute::new("digest", DIGEST_A),
                Attribute::new("owner", "alice"),
            ]
        );

        let record: RecordResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetRecordByDigest {
                    digest: DIGEST_A.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(record.id, 1);
        assert_eq!(record.digest, DIGEST_A);
        assert_eq!(record.owner, "alice");
        assert_eq!(record.label, "service-agreement.pdf");
        assert_eq!(record.memo, Some("Signed PDF hash".to_string()));
        assert!(record.active);
    }

    #[test]
    fn rejects_duplicate_digest() {
        let mut deps = init();
        let msg = ExecuteMsg::Register {
            digest: DIGEST_A.to_string(),
            label: "first".to_string(),
            memo: None,
        };
        execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap();

        let duplicate = ExecuteMsg::Register {
            digest: DIGEST_A.to_uppercase(),
            label: "duplicate".to_string(),
            memo: None,
        };
        let err = execute(deps.as_mut(), mock_env(), mock_info("bob", &[]), duplicate).unwrap_err();

        match err {
            ContractError::DuplicateDigest {} => {}
            _ => panic!("expected duplicate digest error"),
        }
    }

    #[test]
    fn only_owner_updates_and_revokes() {
        let mut deps = init();
        let msg = ExecuteMsg::Register {
            digest: DIGEST_A.to_string(),
            label: "artifact".to_string(),
            memo: None,
        };
        execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::UpdateMemo {
                id: 1,
                memo: Some("not yours".to_string()),
            },
        )
        .unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            _ => panic!("expected unauthorized error"),
        }

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateMemo {
                id: 1,
                memo: Some("new note".to_string()),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Revoke { id: 1 },
        )
        .unwrap();

        let record: RecordResponse =
            from_json(&query(deps.as_ref(), mock_env(), QueryMsg::GetRecord { id: 1 }).unwrap())
                .unwrap();
        assert_eq!(record.memo, Some("new note".to_string()));
        assert!(!record.active);
    }

    #[test]
    fn validates_input() {
        let mut deps = init();
        let invalid_digest = ExecuteMsg::Register {
            digest: "abc".to_string(),
            label: "artifact".to_string(),
            memo: None,
        };
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            invalid_digest,
        )
        .unwrap_err();
        match err {
            ContractError::InvalidDigest {} => {}
            _ => panic!("expected invalid digest error"),
        }

        let empty_label = ExecuteMsg::Register {
            digest: DIGEST_A.to_string(),
            label: " ".to_string(),
            memo: None,
        };
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            empty_label,
        )
        .unwrap_err();
        match err {
            ContractError::InvalidLabel {} => {}
            _ => panic!("expected invalid label error"),
        }

        let long_memo = ExecuteMsg::Register {
            digest: DIGEST_A.to_string(),
            label: "artifact".to_string(),
            memo: Some("x".repeat(MEMO_MAX_LEN + 1)),
        };
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            long_memo,
        )
        .unwrap_err();
        match err {
            ContractError::InvalidMemo {} => {}
            _ => panic!("expected invalid memo error"),
        }
    }

    #[test]
    fn lists_records_with_pagination() {
        let mut deps = init();
        for (digest, label) in [(DIGEST_A, "first"), (DIGEST_B, "second")] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info("alice", &[]),
                ExecuteMsg::Register {
                    digest: digest.to_string(),
                    label: label.to_string(),
                    memo: None,
                },
            )
            .unwrap();
        }

        let list: ListRecordsResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListRecords {
                    start_after: Some(1),
                    limit: Some(10),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(list.records.len(), 1);
        assert_eq!(list.records[0].id, 2);

        let state: StateResponse =
            from_json(&query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap()).unwrap();
        assert_eq!(state.total_records, 2);
    }
}
