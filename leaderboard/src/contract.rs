#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PlayerCountResponse, QueryMsg, ScoreResponse,
    TopScoresResponse,
};
use crate::state::{Config, ScoreEntry, CONFIG, PLAYER_COUNT, SCORES};

const CONTRACT_NAME: &str = "crates.io:leaderboard";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_NAME_LENGTH: u32 = 64;
const DEFAULT_MAX_METADATA_LENGTH: u32 = 160;
const DEFAULT_LIMIT: u32 = 10;
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
        max_name_length: msg.max_name_length.unwrap_or(DEFAULT_MAX_NAME_LENGTH),
        max_metadata_length: msg
            .max_metadata_length
            .unwrap_or(DEFAULT_MAX_METADATA_LENGTH),
    };
    validate_limits(config.max_name_length, config.max_metadata_length)?;

    CONFIG.save(deps.storage, &config)?;
    PLAYER_COUNT.save(deps.storage, &0)?;

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
        ExecuteMsg::SubmitScore {
            player,
            display_name,
            score,
            metadata,
        } => submit_score(deps, env, info, player, display_name, score, metadata),
        ExecuteMsg::RemoveScore { player } => remove_score(deps, info, player),
        ExecuteMsg::UpdateConfig {
            owner,
            max_name_length,
            max_metadata_length,
        } => update_config(deps, info, owner, max_name_length, max_metadata_length),
    }
}

pub fn submit_score(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    player: Option<String>,
    display_name: String,
    score: u64,
    metadata: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let player_addr = match player {
        Some(player) => {
            if info.sender != config.owner {
                return Err(ContractError::Unauthorized {});
            }
            deps.api.addr_validate(&player)?
        }
        None => info.sender.clone(),
    };

    validate_display_name(&display_name, config.max_name_length)?;
    validate_metadata(metadata.as_deref(), config.max_metadata_length)?;

    let existing = SCORES.may_load(deps.storage, &player_addr)?;
    if let Some(current) = existing.as_ref() {
        if score <= current.score {
            return Err(ContractError::ScoreNotHighEnough {});
        }
    } else {
        PLAYER_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;
    }

    let entry = ScoreEntry {
        player: player_addr.clone(),
        display_name,
        score,
        metadata,
        updated_at: env.block.height,
    };
    SCORES.save(deps.storage, &player_addr, &entry)?;

    Ok(Response::new()
        .add_attribute("action", "submit_score")
        .add_attribute("player", player_addr)
        .add_attribute("score", score.to_string()))
}

pub fn remove_score(
    deps: DepsMut,
    info: MessageInfo,
    player: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let player_addr = deps.api.addr_validate(&player)?;

    if info.sender != config.owner && info.sender != player_addr {
        return Err(ContractError::Unauthorized {});
    }

    if SCORES.may_load(deps.storage, &player_addr)?.is_none() {
        return Err(ContractError::ScoreNotFound {});
    }

    SCORES.remove(deps.storage, &player_addr);
    PLAYER_COUNT.update(deps.storage, |count| -> StdResult<_> {
        Ok(count.saturating_sub(1))
    })?;

    Ok(Response::new()
        .add_attribute("action", "remove_score")
        .add_attribute("player", player_addr))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    max_name_length: Option<u32>,
    max_metadata_length: Option<u32>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(max_name_length) = max_name_length {
            config.max_name_length = max_name_length;
        }
        if let Some(max_metadata_length) = max_metadata_length {
            config.max_metadata_length = max_metadata_length;
        }
        validate_limits(config.max_name_length, config.max_metadata_length)?;

        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Score { player } => to_json_binary(&query_score(deps, player)?),
        QueryMsg::TopScores { start_after, limit } => {
            to_json_binary(&query_top_scores(deps, start_after, limit)?)
        }
        QueryMsg::PlayerCount {} => to_json_binary(&query_player_count(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        max_name_length: config.max_name_length,
        max_metadata_length: config.max_metadata_length,
    })
}

pub fn query_score(deps: Deps, player: String) -> StdResult<ScoreResponse> {
    let player_addr = deps.api.addr_validate(&player)?;
    let entry = SCORES.load(deps.storage, &player_addr)?;
    Ok(score_response(entry))
}

pub fn query_top_scores(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TopScoresResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after
        .map(|addr| deps.api.addr_validate(&addr))
        .transpose()?
        .map(|addr| Bound::ExclusiveRaw(addr.as_bytes().to_vec()));

    let mut entries = SCORES
        .range(deps.storage, start, None, Order::Ascending)
        .map(|item| item.map(|(_, entry)| entry))
        .collect::<StdResult<Vec<_>>>()?;

    entries.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.player.as_str().cmp(b.player.as_str()))
    });

    let scores = entries
        .into_iter()
        .take(limit)
        .map(score_response)
        .collect();
    Ok(TopScoresResponse { scores })
}

pub fn query_player_count(deps: Deps) -> StdResult<PlayerCountResponse> {
    let count = PLAYER_COUNT.load(deps.storage)?;
    Ok(PlayerCountResponse { count })
}

fn score_response(entry: ScoreEntry) -> ScoreResponse {
    ScoreResponse {
        player: entry.player.to_string(),
        display_name: entry.display_name,
        score: entry.score,
        metadata: entry.metadata,
        updated_at: entry.updated_at,
    }
}

fn validate_limits(max_name_length: u32, max_metadata_length: u32) -> Result<(), ContractError> {
    if max_name_length == 0 || max_metadata_length == 0 {
        return Err(ContractError::InvalidLimit {});
    }
    Ok(())
}

fn validate_display_name(display_name: &str, max_name_length: u32) -> Result<(), ContractError> {
    if display_name.trim().is_empty() {
        return Err(ContractError::EmptyDisplayName {});
    }
    if display_name.len() > max_name_length as usize {
        return Err(ContractError::DisplayNameTooLong {});
    }
    Ok(())
}

fn validate_metadata(
    metadata: Option<&str>,
    max_metadata_length: u32,
) -> Result<(), ContractError> {
    if metadata
        .map(|value| value.len() > max_metadata_length as usize)
        .unwrap_or(false)
    {
        return Err(ContractError::MetadataTooLong {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    fn instantiate_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            max_name_length: None,
            max_metadata_length: None,
        };
        let info = mock_info("owner", &[]);
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn instantiate_and_query_config() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!("owner", config.owner);
        assert_eq!(DEFAULT_MAX_NAME_LENGTH, config.max_name_length);
        assert_eq!(DEFAULT_MAX_METADATA_LENGTH, config.max_metadata_length);
    }

    #[test]
    fn player_submits_and_queries_score() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let msg = ExecuteMsg::SubmitScore {
            player: None,
            display_name: "Alice".to_string(),
            score: 42,
            metadata: Some("season-one".to_string()),
        };
        execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Score {
                player: "alice".to_string(),
            },
        )
        .unwrap();
        let score: ScoreResponse = from_json(&res).unwrap();
        assert_eq!("alice", score.player);
        assert_eq!("Alice", score.display_name);
        assert_eq!(42, score.score);
        assert_eq!(Some("season-one".to_string()), score.metadata);
    }

    #[test]
    fn lower_score_does_not_replace_best_score() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let submit = |deps: DepsMut, score| {
            execute(
                deps,
                mock_env(),
                mock_info("alice", &[]),
                ExecuteMsg::SubmitScore {
                    player: None,
                    display_name: "Alice".to_string(),
                    score,
                    metadata: None,
                },
            )
        };

        submit(deps.as_mut(), 100).unwrap();
        let err = submit(deps.as_mut(), 99).unwrap_err();
        assert_eq!(ContractError::ScoreNotHighEnough {}, err);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Score {
                player: "alice".to_string(),
            },
        )
        .unwrap();
        let score: ScoreResponse = from_json(&res).unwrap();
        assert_eq!(100, score.score);
    }

    #[test]
    fn top_scores_are_sorted_by_score() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        for (player, display_name, score) in [
            ("alice", "Alice", 10),
            ("bob", "Bob", 25),
            ("carol", "Carol", 18),
        ] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(player, &[]),
                ExecuteMsg::SubmitScore {
                    player: None,
                    display_name: display_name.to_string(),
                    score,
                    metadata: None,
                },
            )
            .unwrap();
        }

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TopScores {
                start_after: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let top: TopScoresResponse = from_json(&res).unwrap();
        assert_eq!(2, top.scores.len());
        assert_eq!("bob", top.scores[0].player);
        assert_eq!(25, top.scores[0].score);
        assert_eq!("carol", top.scores[1].player);
        assert_eq!(18, top.scores[1].score);
    }

    #[test]
    fn owner_can_submit_for_player_and_update_config() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::SubmitScore {
                player: Some("alice".to_string()),
                display_name: "Alice".to_string(),
                score: 55,
                metadata: None,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                owner: Some("new-owner".to_string()),
                max_name_length: Some(32),
                max_metadata_length: Some(64),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!("new-owner", config.owner);
        assert_eq!(32, config.max_name_length);
        assert_eq!(64, config.max_metadata_length);
    }

    #[test]
    fn unauthorized_actions_are_rejected() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SubmitScore {
                player: Some("bob".to_string()),
                display_name: "Bob".to_string(),
                score: 10,
                metadata: None,
            },
        )
        .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, err);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateConfig {
                owner: None,
                max_name_length: Some(10),
                max_metadata_length: None,
            },
        )
        .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, err);
    }

    #[test]
    fn remove_score_updates_player_count() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SubmitScore {
                player: None,
                display_name: "Alice".to_string(),
                score: 7,
                metadata: None,
            },
        )
        .unwrap();

        let count: PlayerCountResponse =
            from_json(&query(deps.as_ref(), mock_env(), QueryMsg::PlayerCount {}).unwrap())
                .unwrap();
        assert_eq!(1, count.count);

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::RemoveScore {
                player: "alice".to_string(),
            },
        )
        .unwrap();

        let count: PlayerCountResponse =
            from_json(&query(deps.as_ref(), mock_env(), QueryMsg::PlayerCount {}).unwrap())
                .unwrap();
        assert_eq!(0, count.count);
    }
}
