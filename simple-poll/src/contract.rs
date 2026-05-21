#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PollOptionResponse, PollResponse, PollsResponse,
    QueryMsg, VoteResponse,
};
use crate::state::{Config, Poll, PollOption, CONFIG, NEXT_POLL_ID, POLLS, VOTES};

const CONTRACT_NAME: &str = "crates.io:simple-poll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_QUESTION_LEN: u16 = 160;
const DEFAULT_MAX_OPTION_LEN: u16 = 64;
const DEFAULT_MAX_OPTIONS: u8 = 8;
const MAX_ALLOWED_QUESTION_LEN: u16 = 500;
const MAX_ALLOWED_OPTION_LEN: u16 = 160;
const MAX_ALLOWED_OPTIONS: u8 = 20;
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
        msg.max_question_len,
        msg.max_option_len,
        msg.max_options,
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    NEXT_POLL_ID.save(deps.storage, &1)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("max_options", config.max_options.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePoll {
            question,
            options,
            closes_at,
        } => execute_create_poll(deps, env, info, question, options, closes_at),
        ExecuteMsg::Vote {
            poll_id,
            option_index,
        } => execute_vote(deps, env, info, poll_id, option_index),
        ExecuteMsg::ClosePoll { poll_id } => execute_close_poll(deps, info, poll_id),
        ExecuteMsg::UpdateConfig {
            max_question_len,
            max_option_len,
            max_options,
            new_owner,
        } => execute_update_config(
            deps,
            info,
            max_question_len,
            max_option_len,
            max_options,
            new_owner,
        ),
    }
}

pub fn execute_create_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    question: String,
    options: Vec<String>,
    closes_at: Option<u64>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let question = validate_question(question, config.max_question_len)?;
    let options = validate_options(options, config.max_option_len, config.max_options)?;

    if let Some(closes_at) = closes_at {
        if closes_at <= env.block.height {
            return Err(ContractError::InvalidCloseHeight {});
        }
    }

    let poll_id = NEXT_POLL_ID.load(deps.storage)?;
    NEXT_POLL_ID.save(deps.storage, &(poll_id + 1))?;

    let poll = Poll {
        id: poll_id,
        creator: info.sender.clone(),
        question,
        options,
        closes_at,
        closed: false,
    };
    POLLS.save(deps.storage, poll_id, &poll)?;

    Ok(Response::new()
        .add_attribute("action", "create_poll")
        .add_attribute("poll_id", poll_id.to_string())
        .add_attribute("creator", info.sender))
}

pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    option_index: u8,
) -> Result<Response, ContractError> {
    let mut poll = POLLS.load(deps.storage, poll_id)?;
    ensure_poll_open(&poll, env.block.height)?;

    if option_index as usize >= poll.options.len() {
        return Err(ContractError::InvalidOptionIndex {});
    }
    if VOTES.has(deps.storage, (poll_id, &info.sender)) {
        return Err(ContractError::AlreadyVoted {});
    }

    poll.options[option_index as usize].votes += 1;
    POLLS.save(deps.storage, poll_id, &poll)?;
    VOTES.save(deps.storage, (poll_id, &info.sender), &option_index)?;

    Ok(Response::new()
        .add_attribute("action", "vote")
        .add_attribute("poll_id", poll_id.to_string())
        .add_attribute("voter", info.sender)
        .add_attribute("option_index", option_index.to_string()))
}

pub fn execute_close_poll(
    deps: DepsMut,
    info: MessageInfo,
    poll_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut poll = POLLS.load(deps.storage, poll_id)?;

    if poll.closed {
        return Err(ContractError::AlreadyClosed {});
    }
    if info.sender != config.owner && info.sender != poll.creator {
        return Err(ContractError::Unauthorized {});
    }

    poll.closed = true;
    POLLS.save(deps.storage, poll_id, &poll)?;

    Ok(Response::new()
        .add_attribute("action", "close_poll")
        .add_attribute("poll_id", poll_id.to_string())
        .add_attribute("closer", info.sender))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    max_question_len: Option<u16>,
    max_option_len: Option<u16>,
    max_options: Option<u8>,
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
        max_question_len.or(Some(config.max_question_len)),
        max_option_len.or(Some(config.max_option_len)),
        max_options.or(Some(config.max_options)),
    )?;
    CONFIG.save(deps.storage, &updated)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", updated.owner)
        .add_attribute("max_options", updated.max_options.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Poll { poll_id } => to_json_binary(&query_poll(deps, poll_id)?),
        QueryMsg::Polls { start_after, limit } => {
            to_json_binary(&query_polls(deps, start_after, limit)?)
        }
        QueryMsg::Vote { poll_id, voter } => to_json_binary(&query_vote(deps, poll_id, voter)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.into_string(),
        max_question_len: config.max_question_len,
        max_option_len: config.max_option_len,
        max_options: config.max_options,
    })
}

pub fn query_poll(deps: Deps, poll_id: u64) -> StdResult<PollResponse> {
    let poll = POLLS.load(deps.storage, poll_id)?;
    Ok(poll_response(poll))
}

pub fn query_polls(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<PollsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let polls = POLLS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|poll| {
            let (_, poll) = poll?;
            Ok(poll_response(poll))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(PollsResponse { polls })
}

pub fn query_vote(deps: Deps, poll_id: u64, voter: String) -> StdResult<VoteResponse> {
    let voter_addr = deps.api.addr_validate(&voter)?;
    let option_index = VOTES.may_load(deps.storage, (poll_id, &voter_addr))?;

    Ok(VoteResponse {
        poll_id,
        voter,
        option_index,
    })
}

fn ensure_poll_open(poll: &Poll, height: u64) -> Result<(), ContractError> {
    if poll.closed
        || poll
            .closes_at
            .map(|closes_at| height >= closes_at)
            .unwrap_or(false)
    {
        return Err(ContractError::PollClosed {});
    }

    Ok(())
}

fn poll_response(poll: Poll) -> PollResponse {
    let total_votes = poll.options.iter().map(|option| option.votes).sum();
    let options = poll
        .options
        .into_iter()
        .enumerate()
        .map(|(index, option)| PollOptionResponse {
            index: index as u8,
            label: option.label,
            votes: option.votes,
        })
        .collect();

    PollResponse {
        id: poll.id,
        creator: poll.creator.into_string(),
        question: poll.question,
        options,
        closes_at: poll.closes_at,
        closed: poll.closed,
        total_votes,
    }
}

fn validate_config(
    owner: Addr,
    max_question_len: Option<u16>,
    max_option_len: Option<u16>,
    max_options: Option<u8>,
) -> Result<Config, ContractError> {
    let max_question_len = max_question_len.unwrap_or(DEFAULT_MAX_QUESTION_LEN);
    if max_question_len == 0 || max_question_len > MAX_ALLOWED_QUESTION_LEN {
        return Err(ContractError::InvalidConfig {});
    }

    let max_option_len = max_option_len.unwrap_or(DEFAULT_MAX_OPTION_LEN);
    if max_option_len == 0 || max_option_len > MAX_ALLOWED_OPTION_LEN {
        return Err(ContractError::InvalidConfig {});
    }

    let max_options = max_options.unwrap_or(DEFAULT_MAX_OPTIONS);
    if !(2..=MAX_ALLOWED_OPTIONS).contains(&max_options) {
        return Err(ContractError::InvalidConfig {});
    }

    Ok(Config {
        owner,
        max_question_len,
        max_option_len,
        max_options,
    })
}

fn validate_question(question: String, max_question_len: u16) -> Result<String, ContractError> {
    let question = question.trim().to_string();
    if question.is_empty() || question.chars().count() > max_question_len as usize {
        return Err(ContractError::InvalidQuestion {});
    }

    Ok(question)
}

fn validate_options(
    options: Vec<String>,
    max_option_len: u16,
    max_options: u8,
) -> Result<Vec<PollOption>, ContractError> {
    if options.len() < 2 || options.len() > max_options as usize {
        return Err(ContractError::InvalidOptionCount {});
    }

    let mut labels: Vec<String> = Vec::with_capacity(options.len());
    for option in options {
        let label = option.trim().to_string();
        if label.is_empty() || label.chars().count() > max_option_len as usize {
            return Err(ContractError::InvalidOptionLabel {});
        }
        if labels.iter().any(|existing| existing == &label) {
            return Err(ContractError::InvalidOptionLabel {});
        }
        labels.push(label);
    }

    Ok(labels
        .into_iter()
        .map(|label| PollOption { label, votes: 0 })
        .collect())
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
                max_question_len: Some(80),
                max_option_len: Some(20),
                max_options: Some(4),
            },
        )
        .unwrap();
        (deps, env)
    }

    fn create_poll(deps: DepsMut, env: Env, creator: &str) {
        execute(
            deps,
            env,
            mock_info(creator, &[]),
            ExecuteMsg::CreatePoll {
                question: "Which feature should ship first?".to_string(),
                options: vec!["Dark mode".to_string(), "Exports".to_string()],
                closes_at: Some(150),
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
        assert_eq!(config.max_question_len, 80);
        assert_eq!(config.max_option_len, 20);
        assert_eq!(config.max_options, 4);
    }

    #[test]
    fn create_poll_trims_and_stores_options() {
        let (mut deps, env) = setup();
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::CreatePoll {
                question: "  Pick a launch plan  ".to_string(),
                options: vec!["  Alpha  ".to_string(), "Beta".to_string()],
                closes_at: None,
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env, QueryMsg::Poll { poll_id: 1 }).unwrap();
        let poll: PollResponse = from_json(&res).unwrap();

        assert_eq!(poll.creator, "creator");
        assert_eq!(poll.question, "Pick a launch plan");
        assert_eq!(poll.options[0].label, "Alpha");
        assert_eq!(poll.options[1].label, "Beta");
        assert_eq!(poll.total_votes, 0);
    }

    #[test]
    fn rejects_invalid_poll_inputs() {
        let (mut deps, env) = setup();
        let duplicate = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::CreatePoll {
                question: "Duplicate options?".to_string(),
                options: vec!["Yes".to_string(), "Yes".to_string()],
                closes_at: None,
            },
        )
        .unwrap_err();
        assert_eq!(duplicate, ContractError::InvalidOptionLabel {});

        let past_close = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::CreatePoll {
                question: "Close in the past?".to_string(),
                options: vec!["Yes".to_string(), "No".to_string()],
                closes_at: Some(100),
            },
        )
        .unwrap_err();
        assert_eq!(past_close, ContractError::InvalidCloseHeight {});
    }

    #[test]
    fn vote_records_one_vote_per_address() {
        let (mut deps, env) = setup();
        create_poll(deps.as_mut(), env.clone(), "creator");

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("voter", &[]),
            ExecuteMsg::Vote {
                poll_id: 1,
                option_index: 1,
            },
        )
        .unwrap();

        let repeat = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("voter", &[]),
            ExecuteMsg::Vote {
                poll_id: 1,
                option_index: 0,
            },
        )
        .unwrap_err();
        assert_eq!(repeat, ContractError::AlreadyVoted {});

        let poll: PollResponse =
            from_json(&query(deps.as_ref(), env.clone(), QueryMsg::Poll { poll_id: 1 }).unwrap())
                .unwrap();
        assert_eq!(poll.options[1].votes, 1);
        assert_eq!(poll.total_votes, 1);

        let vote: VoteResponse = from_json(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::Vote {
                    poll_id: 1,
                    voter: "voter".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(vote.option_index, Some(1));
    }

    #[test]
    fn rejects_invalid_option_index() {
        let (mut deps, env) = setup();
        create_poll(deps.as_mut(), env.clone(), "creator");

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("voter", &[]),
            ExecuteMsg::Vote {
                poll_id: 1,
                option_index: 2,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::InvalidOptionIndex {});
    }

    #[test]
    fn rejects_votes_at_or_after_close_height() {
        let (mut deps, mut env) = setup();
        create_poll(deps.as_mut(), env.clone(), "creator");
        env.block.height = 150;

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("late_voter", &[]),
            ExecuteMsg::Vote {
                poll_id: 1,
                option_index: 0,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::PollClosed {});
    }

    #[test]
    fn creator_or_owner_can_close_poll() {
        let (mut deps, env) = setup();
        create_poll(deps.as_mut(), env.clone(), "creator");

        let unauthorized = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("stranger", &[]),
            ExecuteMsg::ClosePoll { poll_id: 1 },
        )
        .unwrap_err();
        assert_eq!(unauthorized, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::ClosePoll { poll_id: 1 },
        )
        .unwrap();

        let poll: PollResponse =
            from_json(&query(deps.as_ref(), env, QueryMsg::Poll { poll_id: 1 }).unwrap()).unwrap();
        assert!(poll.closed);
    }

    #[test]
    fn owner_updates_config_and_owner() {
        let (mut deps, env) = setup();
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                max_question_len: Some(120),
                max_option_len: Some(30),
                max_options: Some(6),
                new_owner: Some("next_admin".to_string()),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_json(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.owner, "next_admin");
        assert_eq!(config.max_question_len, 120);
        assert_eq!(config.max_option_len, 30);
        assert_eq!(config.max_options, 6);
    }

    #[test]
    fn list_polls_uses_start_after_and_limit() {
        let (mut deps, env) = setup();
        create_poll(deps.as_mut(), env.clone(), "creator");
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("creator", &[]),
            ExecuteMsg::CreatePoll {
                question: "Second poll?".to_string(),
                options: vec!["One".to_string(), "Two".to_string()],
                closes_at: None,
            },
        )
        .unwrap();

        let polls: PollsResponse = from_json(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::Polls {
                    start_after: Some(1),
                    limit: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(polls.polls.len(), 1);
        assert_eq!(polls.polls[0].id, 2);
    }
}
