#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    AwardInfo, BadgeInfo, BadgeResponse, BadgesResponse, ConfigResponse, ExecuteMsg,
    HasBadgeResponse, HolderBadgesResponse, InstantiateMsg, IsIssuerResponse, MigrateMsg, QueryMsg,
};
use crate::state::{Award, Badge, Config, AWARDS, BADGES, CONFIG, HOLDER_COUNTS, ISSUERS};

const CONTRACT_NAME: &str = "crates.io:achievement-badges";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_BADGES_PER_HOLDER: u32 = 25;
const MAX_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let name = validate_text("name", msg.name, 3, 64)?;
    let description = msg
        .description
        .map(|value| validate_text("description", value, 1, 160))
        .transpose()?;
    let max_badges_per_holder = msg
        .max_badges_per_holder
        .unwrap_or(DEFAULT_MAX_BADGES_PER_HOLDER);

    if max_badges_per_holder == 0 {
        return Err(ContractError::InvalidInput {
            reason: "max_badges_per_holder must be greater than zero".to_string(),
        });
    }

    let config = Config {
        owner: info.sender.clone(),
        name,
        description,
        max_badges_per_holder,
    };

    CONFIG.save(deps.storage, &config)?;
    ISSUERS.save(deps.storage, &info.sender, &true)?;

    for issuer in msg.issuers {
        let issuer = deps.api.addr_validate(&issuer)?;
        ISSUERS.save(deps.storage, &issuer, &true)?;
    }

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
        ExecuteMsg::CreateBadge {
            badge_id,
            title,
            description,
        } => create_badge(deps, info, badge_id, title, description),
        ExecuteMsg::ArchiveBadge { badge_id } => archive_badge(deps, info, badge_id),
        ExecuteMsg::AddIssuer { issuer } => add_issuer(deps, info, issuer),
        ExecuteMsg::RemoveIssuer { issuer } => remove_issuer(deps, info, issuer),
        ExecuteMsg::AwardBadge {
            badge_id,
            recipient,
            note,
        } => award_badge(deps, env, info, badge_id, recipient, note),
        ExecuteMsg::RevokeBadge { badge_id, holder } => revoke_badge(deps, info, badge_id, holder),
        ExecuteMsg::TransferOwnership { new_owner } => transfer_ownership(deps, info, new_owner),
    }
}

pub fn create_badge(
    deps: DepsMut,
    info: MessageInfo,
    badge_id: String,
    title: String,
    description: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;

    let badge_id = validate_id(badge_id)?;
    let title = validate_text("title", title, 3, 64)?;
    let description = validate_text("description", description, 3, 200)?;

    if BADGES.has(deps.storage, badge_id.as_str()) {
        return Err(ContractError::BadgeExists {});
    }

    let badge = Badge {
        badge_id: badge_id.clone(),
        title,
        description,
        creator: info.sender.clone(),
        archived: false,
    };

    BADGES.save(deps.storage, badge_id.as_str(), &badge)?;

    Ok(Response::new()
        .add_attribute("action", "create_badge")
        .add_attribute("badge_id", badge_id)
        .add_attribute("creator", info.sender))
}

pub fn archive_badge(
    deps: DepsMut,
    info: MessageInfo,
    badge_id: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;
    let badge_id = validate_id(badge_id)?;

    BADGES.update(
        deps.storage,
        badge_id.as_str(),
        |badge| -> Result<Badge, ContractError> {
            let mut badge = badge.ok_or(ContractError::BadgeNotFound {})?;
            badge.archived = true;
            Ok(badge)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "archive_badge")
        .add_attribute("badge_id", badge_id))
}

pub fn add_issuer(
    deps: DepsMut,
    info: MessageInfo,
    issuer: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;
    let issuer = deps.api.addr_validate(&issuer)?;
    ISSUERS.save(deps.storage, &issuer, &true)?;

    Ok(Response::new()
        .add_attribute("action", "add_issuer")
        .add_attribute("issuer", issuer))
}

pub fn remove_issuer(
    deps: DepsMut,
    info: MessageInfo,
    issuer: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;
    let issuer = deps.api.addr_validate(&issuer)?;
    let config = CONFIG.load(deps.storage)?;

    if issuer == config.owner {
        return Err(ContractError::CannotRemoveOwnerIssuer {});
    }

    ISSUERS.remove(deps.storage, &issuer);

    Ok(Response::new()
        .add_attribute("action", "remove_issuer")
        .add_attribute("issuer", issuer))
}

pub fn award_badge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    badge_id: String,
    recipient: String,
    note: Option<String>,
) -> Result<Response, ContractError> {
    assert_issuer(deps.as_ref(), &info.sender)?;

    let badge_id = validate_id(badge_id)?;
    let recipient = deps.api.addr_validate(&recipient)?;
    let note = note
        .map(|value| validate_text("note", value, 1, 120))
        .transpose()?;

    let config = CONFIG.load(deps.storage)?;
    let badge = BADGES
        .may_load(deps.storage, badge_id.as_str())?
        .ok_or(ContractError::BadgeNotFound {})?;

    if badge.archived {
        return Err(ContractError::BadgeArchived {});
    }

    if AWARDS.has(deps.storage, (&recipient, badge_id.as_str())) {
        return Err(ContractError::AwardExists {});
    }

    let count = HOLDER_COUNTS
        .may_load(deps.storage, &recipient)?
        .unwrap_or_default();

    if count >= config.max_badges_per_holder {
        return Err(ContractError::LimitReached {});
    }

    let award = Award {
        badge_id: badge_id.clone(),
        holder: recipient.clone(),
        awarded_by: info.sender.clone(),
        awarded_at_height: env.block.height,
        note,
    };

    AWARDS.save(deps.storage, (&recipient, badge_id.as_str()), &award)?;
    HOLDER_COUNTS.save(deps.storage, &recipient, &(count + 1))?;

    Ok(Response::new()
        .add_attribute("action", "award_badge")
        .add_attribute("badge_id", badge_id)
        .add_attribute("holder", recipient)
        .add_attribute("awarded_by", info.sender))
}

pub fn revoke_badge(
    deps: DepsMut,
    info: MessageInfo,
    badge_id: String,
    holder: String,
) -> Result<Response, ContractError> {
    let badge_id = validate_id(badge_id)?;
    let holder = deps.api.addr_validate(&holder)?;
    let config = CONFIG.load(deps.storage)?;
    let award = AWARDS
        .may_load(deps.storage, (&holder, badge_id.as_str()))?
        .ok_or(ContractError::AwardNotFound {})?;

    if info.sender != config.owner && info.sender != award.awarded_by {
        return Err(ContractError::Unauthorized {});
    }

    AWARDS.remove(deps.storage, (&holder, badge_id.as_str()));
    let count = HOLDER_COUNTS
        .may_load(deps.storage, &holder)?
        .unwrap_or_default()
        .saturating_sub(1);
    HOLDER_COUNTS.save(deps.storage, &holder, &count)?;

    Ok(Response::new()
        .add_attribute("action", "revoke_badge")
        .add_attribute("badge_id", badge_id)
        .add_attribute("holder", holder))
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
    ISSUERS.save(deps.storage, &new_owner, &true)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_attribute("new_owner", new_owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::IsIssuer { address } => to_json_binary(&query_is_issuer(deps, address)?),
        QueryMsg::Badge { badge_id } => to_json_binary(&query_badge(deps, badge_id)?),
        QueryMsg::Badges { start_after, limit } => {
            to_json_binary(&query_badges(deps, start_after, limit)?)
        }
        QueryMsg::HolderBadges {
            holder,
            start_after,
            limit,
        } => to_json_binary(&query_holder_badges(deps, holder, start_after, limit)?),
        QueryMsg::HasBadge { holder, badge_id } => {
            to_json_binary(&query_has_badge(deps, holder, badge_id)?)
        }
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
        name: config.name,
        description: config.description,
        max_badges_per_holder: config.max_badges_per_holder,
    })
}

fn query_is_issuer(deps: Deps, address: String) -> StdResult<IsIssuerResponse> {
    let address = deps.api.addr_validate(&address)?;
    let is_issuer = ISSUERS
        .may_load(deps.storage, &address)?
        .unwrap_or_default();

    Ok(IsIssuerResponse {
        address: address.to_string(),
        is_issuer,
    })
}

fn query_badge(deps: Deps, badge_id: String) -> StdResult<BadgeResponse> {
    let badge_id = validate_id_std(badge_id)?;
    let badge = BADGES.load(deps.storage, badge_id.as_str())?;
    Ok(BadgeResponse {
        badge: badge_to_info(badge),
    })
}

fn query_badges(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<BadgesResponse> {
    let limit = limit.unwrap_or(20).min(MAX_LIMIT) as usize;
    let start = start_after.as_deref().map(Bound::exclusive);
    let badges = BADGES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, badge)| badge_to_info(badge)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BadgesResponse { badges })
}

fn query_holder_badges(
    deps: Deps,
    holder: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<HolderBadgesResponse> {
    let holder = deps.api.addr_validate(&holder)?;
    let limit = limit.unwrap_or(20).min(MAX_LIMIT) as usize;
    let start = start_after.as_deref().map(Bound::exclusive);
    let awards = AWARDS
        .prefix(&holder)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, award)| award_to_info(award)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(HolderBadgesResponse {
        holder: holder.to_string(),
        awards,
    })
}

fn query_has_badge(deps: Deps, holder: String, badge_id: String) -> StdResult<HasBadgeResponse> {
    let holder = deps.api.addr_validate(&holder)?;
    let badge_id = validate_id_std(badge_id)?;
    let award = AWARDS
        .may_load(deps.storage, (&holder, badge_id.as_str()))?
        .map(award_to_info);

    Ok(HasBadgeResponse {
        has_badge: award.is_some(),
        award,
    })
}

fn assert_owner(deps: Deps, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != *sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn assert_issuer(deps: Deps, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    let is_issuer = ISSUERS.may_load(deps.storage, sender)?.unwrap_or_default();

    if !is_issuer {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

fn badge_to_info(badge: Badge) -> BadgeInfo {
    BadgeInfo {
        badge_id: badge.badge_id,
        title: badge.title,
        description: badge.description,
        creator: badge.creator.to_string(),
        archived: badge.archived,
    }
}

fn award_to_info(award: Award) -> AwardInfo {
    AwardInfo {
        badge_id: award.badge_id,
        holder: award.holder.to_string(),
        awarded_by: award.awarded_by.to_string(),
        awarded_at_height: award.awarded_at_height,
        note: award.note,
    }
}

fn validate_id(value: String) -> Result<String, ContractError> {
    let value = value.trim().to_ascii_lowercase();

    if value.len() < 3 || value.len() > 48 {
        return Err(ContractError::InvalidInput {
            reason: "badge_id must be 3 to 48 characters".to_string(),
        });
    }

    if !value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(ContractError::InvalidInput {
            reason: "badge_id may only contain lowercase letters, digits, and hyphens".to_string(),
        });
    }

    Ok(value)
}

fn validate_id_std(value: String) -> StdResult<String> {
    validate_id(value).map_err(|err| cosmwasm_std::StdError::generic_err(err.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env, mock_info},
    };

    fn instantiate_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            name: "Learning Badges".to_string(),
            description: Some("Badge examples".to_string()),
            issuers: vec!["issuer".to_string()],
            max_badges_per_holder: Some(2),
        };
        instantiate(deps, mock_env(), mock_info("owner", &[]), msg).unwrap();
    }

    fn create_first_badge(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CreateBadge {
                badge_id: "first-contract".to_string(),
                title: "First Contract".to_string(),
                description: "Completed a first contract".to_string(),
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

        assert_eq!(config.owner, "owner");
        assert_eq!(config.name, "Learning Badges");
        assert_eq!(config.max_badges_per_holder, 2);
    }

    #[test]
    fn owner_creates_badge_and_lists_it() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_first_badge(deps.as_mut());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Badges {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let badges: BadgesResponse = from_json(&res).unwrap();

        assert_eq!(badges.badges.len(), 1);
        assert_eq!(badges.badges[0].badge_id, "first-contract");
        assert!(!badges.badges[0].archived);
    }

    #[test]
    fn issuer_awards_badge_and_holder_can_query_it() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_first_badge(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("issuer", &[]),
            ExecuteMsg::AwardBadge {
                badge_id: "first-contract".to_string(),
                recipient: "learner".to_string(),
                note: Some("Finished lesson one".to_string()),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::HolderBadges {
                holder: "learner".to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let holder_badges: HolderBadgesResponse = from_json(&res).unwrap();

        assert_eq!(holder_badges.holder, "learner");
        assert_eq!(holder_badges.awards.len(), 1);
        assert_eq!(holder_badges.awards[0].awarded_by, "issuer");

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::HasBadge {
                holder: "learner".to_string(),
                badge_id: "first-contract".to_string(),
            },
        )
        .unwrap();
        let has_badge: HasBadgeResponse = from_json(&res).unwrap();

        assert!(has_badge.has_badge);
    }

    #[test]
    fn unauthorized_sender_cannot_create_or_award() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_first_badge(deps.as_mut());

        let create_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::CreateBadge {
                badge_id: "second-contract".to_string(),
                title: "Second Contract".to_string(),
                description: "Another badge".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(create_err, ContractError::Unauthorized {});

        let award_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::AwardBadge {
                badge_id: "first-contract".to_string(),
                recipient: "learner".to_string(),
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(award_err, ContractError::Unauthorized {});
    }

    #[test]
    fn duplicate_awards_and_holder_limits_are_rejected() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_first_badge(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CreateBadge {
                badge_id: "second-contract".to_string(),
                title: "Second Contract".to_string(),
                description: "Completed a second contract".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CreateBadge {
                badge_id: "third-contract".to_string(),
                title: "Third Contract".to_string(),
                description: "Completed a third contract".to_string(),
            },
        )
        .unwrap();

        for badge_id in ["first-contract", "second-contract"] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info("issuer", &[]),
                ExecuteMsg::AwardBadge {
                    badge_id: badge_id.to_string(),
                    recipient: "learner".to_string(),
                    note: None,
                },
            )
            .unwrap();
        }

        let duplicate_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("issuer", &[]),
            ExecuteMsg::AwardBadge {
                badge_id: "first-contract".to_string(),
                recipient: "learner".to_string(),
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(duplicate_err, ContractError::AwardExists {});

        let limit_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("issuer", &[]),
            ExecuteMsg::AwardBadge {
                badge_id: "third-contract".to_string(),
                recipient: "learner".to_string(),
                note: None,
            },
        )
        .unwrap_err();
        assert_eq!(limit_err, ContractError::LimitReached {});
    }

    #[test]
    fn owner_archives_and_award_issuer_can_revoke() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_first_badge(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("issuer", &[]),
            ExecuteMsg::AwardBadge {
                badge_id: "first-contract".to_string(),
                recipient: "learner".to_string(),
                note: None,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("issuer", &[]),
            ExecuteMsg::RevokeBadge {
                badge_id: "first-contract".to_string(),
                holder: "learner".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::HasBadge {
                holder: "learner".to_string(),
                badge_id: "first-contract".to_string(),
            },
        )
        .unwrap();
        let has_badge: HasBadgeResponse = from_json(&res).unwrap();
        assert!(!has_badge.has_badge);

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::ArchiveBadge {
                badge_id: "first-contract".to_string(),
            },
        )
        .unwrap();

        let archived_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("issuer", &[]),
            ExecuteMsg::AwardBadge {
                badge_id: "first-contract".to_string(),
                recipient: "learner".to_string(),
                note: None,
            },
        )
        .unwrap_err();

        assert_eq!(archived_err, ContractError::BadgeArchived {});
    }
}
