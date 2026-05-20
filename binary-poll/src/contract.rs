#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse};
use crate::state::{Poll, POLL, VOTES};

const CONTRACT_NAME: &str = "crates.io:binary-poll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let question = msg.question.trim();
    if question.is_empty() {
        return Err(ContractError::EmptyQuestion {});
    }
    if msg.closes_at_height <= env.block.height {
        return Err(ContractError::InvalidCloseHeight {});
    }

    POLL.save(
        deps.storage,
        &Poll {
            creator: info.sender.clone(),
            question: question.to_string(),
            closes_at_height: msg.closes_at_height,
            yes_votes: 0,
            no_votes: 0,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender)
        .add_attribute("closes_at_height", msg.closes_at_height.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Vote { approve } => execute_vote(deps, env, info, approve),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Poll {} => to_json_binary(&query_poll(deps, env)?),
        QueryMsg::Vote { voter } => to_json_binary(&query_vote(deps, voter)?),
    }
}

fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    approve: bool,
) -> Result<Response, ContractError> {
    let mut poll = POLL.load(deps.storage)?;
    if env.block.height >= poll.closes_at_height {
        return Err(ContractError::PollClosed {});
    }
    if VOTES.may_load(deps.storage, &info.sender)?.is_some() {
        return Err(ContractError::AlreadyVoted {});
    }

    if approve {
        poll.yes_votes += 1;
    } else {
        poll.no_votes += 1;
    }

    POLL.save(deps.storage, &poll)?;
    VOTES.save(deps.storage, &info.sender, &approve)?;

    Ok(Response::new()
        .add_attribute("action", "vote")
        .add_attribute("voter", info.sender)
        .add_attribute("approve", approve.to_string()))
}

fn query_poll(deps: Deps, env: Env) -> StdResult<PollResponse> {
    let poll = POLL.load(deps.storage)?;
    Ok(PollResponse {
        creator: poll.creator.to_string(),
        question: poll.question,
        closes_at_height: poll.closes_at_height,
        yes_votes: poll.yes_votes,
        no_votes: poll.no_votes,
        open: env.block.height < poll.closes_at_height,
        result: result_label(poll.yes_votes, poll.no_votes).to_string(),
    })
}

fn query_vote(deps: Deps, voter: String) -> StdResult<VoteResponse> {
    let voter_addr = deps.api.addr_validate(&voter)?;
    let approve = VOTES.may_load(deps.storage, &voter_addr)?;
    Ok(VoteResponse {
        voter,
        voted: approve.is_some(),
        approve,
    })
}

fn result_label(yes_votes: u64, no_votes: u64) -> &'static str {
    if yes_votes > no_votes {
        "yes"
    } else if no_votes > yes_votes {
        "no"
    } else {
        "tie"
    }
}
