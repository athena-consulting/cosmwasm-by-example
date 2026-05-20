#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    AnnouncementResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, ListAnnouncementsResponse,
    QueryMsg,
};
use crate::state::{Announcement, Config, ANNOUNCEMENTS, CONFIG};

const CONTRACT_NAME: &str = "crates.io:expiring-announcements";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        },
    )?;

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
        ExecuteMsg::Publish {
            id,
            title,
            body,
            expires_at,
        } => execute_publish(deps, env, info, id, title, body, expires_at),
        ExecuteMsg::Archive { id } => execute_archive(deps, info, id),
        ExecuteMsg::TransferOwnership { new_owner } => {
            execute_transfer_ownership(deps, info, new_owner)
        }
    }
}

fn assert_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn execute_publish(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    title: String,
    body: String,
    expires_at: u64,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info)?;

    let id = id.trim().to_string();
    let title = title.trim().to_string();
    let body = body.trim().to_string();
    if id.is_empty() || title.is_empty() || body.is_empty() {
        return Err(ContractError::EmptyField {});
    }
    if expires_at <= env.block.height {
        return Err(ContractError::InvalidExpiration {});
    }
    if ANNOUNCEMENTS.has(deps.storage, &id) {
        return Err(ContractError::AnnouncementExists {});
    }

    let announcement = Announcement {
        id: id.clone(),
        title,
        body,
        expires_at,
        archived: false,
    };
    ANNOUNCEMENTS.save(deps.storage, &id, &announcement)?;

    Ok(Response::new()
        .add_attribute("action", "publish")
        .add_attribute("id", id)
        .add_attribute("expires_at", expires_at.to_string()))
}

fn execute_archive(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info)?;
    ANNOUNCEMENTS.update(deps.storage, &id, |record| -> Result<_, ContractError> {
        let mut announcement = record.ok_or(ContractError::AnnouncementNotFound {})?;
        announcement.archived = true;
        Ok(announcement)
    })?;

    Ok(Response::new()
        .add_attribute("action", "archive")
        .add_attribute("id", id))
}

fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info)?;
    let new_owner_addr = deps.api.addr_validate(new_owner.trim())?;
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        config.owner = new_owner_addr.clone();
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_attribute("new_owner", new_owner_addr))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Announcement { id } => to_json_binary(&query_announcement(deps, env, id)?),
        QueryMsg::List { active_only } => {
            to_json_binary(&query_list(deps, env, active_only.unwrap_or(false))?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
    })
}

fn query_announcement(deps: Deps, env: Env, id: String) -> StdResult<AnnouncementResponse> {
    let announcement = ANNOUNCEMENTS.load(deps.storage, &id)?;
    let active = is_active(&announcement, env.block.height);
    Ok(AnnouncementResponse {
        announcement,
        active,
    })
}

fn query_list(deps: Deps, env: Env, active_only: bool) -> StdResult<ListAnnouncementsResponse> {
    let announcements = ANNOUNCEMENTS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| match item {
            Ok((_id, announcement)) => {
                let active = is_active(&announcement, env.block.height);
                if active_only && !active {
                    None
                } else {
                    Some(Ok(AnnouncementResponse {
                        announcement,
                        active,
                    }))
                }
            }
            Err(err) => Some(Err(err)),
        })
        .collect::<StdResult<Vec<_>>>()?;
    Ok(ListAnnouncementsResponse { announcements })
}

fn is_active(announcement: &Announcement, block_height: u64) -> bool {
    !announcement.archived && block_height < announcement.expires_at
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    fn init() -> cosmwasm_std::OwnedDeps<
        cosmwasm_std::MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    > {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg {},
        )
        .unwrap();
        deps
    }

    fn publish(id: &str, expires_at: u64) -> ExecuteMsg {
        ExecuteMsg::Publish {
            id: id.to_string(),
            title: format!("Title {id}"),
            body: format!("Body {id}"),
            expires_at,
        }
    }

    #[test]
    fn owner_can_publish_and_query_active_announcement() {
        let mut deps = init();
        let mut env = mock_env();
        env.block.height = 10;

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            publish("maintenance", 20),
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Announcement {
                id: "maintenance".to_string(),
            },
        )
        .unwrap();
        let value: AnnouncementResponse = from_json(&res).unwrap();
        assert_eq!(value.announcement.id, "maintenance");
        assert!(value.active);
    }

    #[test]
    fn rejects_unauthorized_or_invalid_publish() {
        let mut deps = init();
        let mut env = mock_env();
        env.block.height = 10;

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("intruder", &[]),
            publish("notice", 20),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            publish("notice", 10),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidExpiration {});

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("owner", &[]),
            ExecuteMsg::Publish {
                id: " ".to_string(),
                title: "title".to_string(),
                body: "body".to_string(),
                expires_at: 20,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::EmptyField {});
    }

    #[test]
    fn blocks_duplicate_ids() {
        let mut deps = init();
        let mut env = mock_env();
        env.block.height = 10;

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            publish("notice", 20),
        )
        .unwrap();
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("owner", &[]),
            publish("notice", 25),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AnnouncementExists {});
    }

    #[test]
    fn active_filter_omits_expired_and_archived_announcements() {
        let mut deps = init();
        let mut env = mock_env();
        env.block.height = 10;

        for (id, expires_at) in [("active", 30), ("expired", 15), ("archived", 30)] {
            execute(
                deps.as_mut(),
                env.clone(),
                mock_info("owner", &[]),
                publish(id, expires_at),
            )
            .unwrap();
        }
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::Archive {
                id: "archived".to_string(),
            },
        )
        .unwrap();

        env.block.height = 20;
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::List {
                active_only: Some(true),
            },
        )
        .unwrap();
        let value: ListAnnouncementsResponse = from_json(&res).unwrap();
        assert_eq!(value.announcements.len(), 1);
        assert_eq!(value.announcements[0].announcement.id, "active");
        assert!(value.announcements[0].active);
    }

    #[test]
    fn ownership_can_be_transferred() {
        let mut deps = init();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::TransferOwnership {
                new_owner: "new-owner".to_string(),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(value.owner, cosmwasm_std::Addr::unchecked("new-owner"));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            publish("old-owner-notice", 20),
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }
}
