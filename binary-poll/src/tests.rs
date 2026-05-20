use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Env;

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse};
use crate::ContractError;

fn env_at(height: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env
}

fn setup_poll() -> cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
> {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        env_at(10),
        mock_info("creator", &[]),
        InstantiateMsg {
            question: "Should the upgrade pass?".to_string(),
            closes_at_height: 20,
        },
    )
    .unwrap();
    deps
}

#[test]
fn counts_yes_and_no_votes_before_close() {
    let mut deps = setup_poll();

    execute(
        deps.as_mut(),
        env_at(11),
        mock_info("alice", &[]),
        ExecuteMsg::Vote { approve: true },
    )
    .unwrap();
    execute(
        deps.as_mut(),
        env_at(12),
        mock_info("bob", &[]),
        ExecuteMsg::Vote { approve: false },
    )
    .unwrap();
    execute(
        deps.as_mut(),
        env_at(13),
        mock_info("carol", &[]),
        ExecuteMsg::Vote { approve: true },
    )
    .unwrap();

    let bin = query(deps.as_ref(), env_at(13), QueryMsg::Poll {}).unwrap();
    let poll: PollResponse = cosmwasm_std::from_json(&bin).unwrap();

    assert_eq!("creator", poll.creator);
    assert_eq!("Should the upgrade pass?", poll.question);
    assert_eq!(2, poll.yes_votes);
    assert_eq!(1, poll.no_votes);
    assert!(poll.open);
    assert_eq!("yes", poll.result);
}

#[test]
fn rejects_duplicate_votes() {
    let mut deps = setup_poll();

    execute(
        deps.as_mut(),
        env_at(11),
        mock_info("alice", &[]),
        ExecuteMsg::Vote { approve: true },
    )
    .unwrap();

    let err = execute(
        deps.as_mut(),
        env_at(12),
        mock_info("alice", &[]),
        ExecuteMsg::Vote { approve: false },
    )
    .unwrap_err();

    assert_eq!(ContractError::AlreadyVoted {}, err);
}

#[test]
fn rejects_votes_after_close_height() {
    let mut deps = setup_poll();

    let err = execute(
        deps.as_mut(),
        env_at(20),
        mock_info("alice", &[]),
        ExecuteMsg::Vote { approve: true },
    )
    .unwrap_err();

    assert_eq!(ContractError::PollClosed {}, err);
}

#[test]
fn validates_instantiation_input() {
    let mut deps = mock_dependencies();

    let err = instantiate(
        deps.as_mut(),
        env_at(10),
        mock_info("creator", &[]),
        InstantiateMsg {
            question: " ".to_string(),
            closes_at_height: 20,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::EmptyQuestion {}, err);

    let err = instantiate(
        deps.as_mut(),
        env_at(10),
        mock_info("creator", &[]),
        InstantiateMsg {
            question: "Ready?".to_string(),
            closes_at_height: 10,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::InvalidCloseHeight {}, err);
}

#[test]
fn queries_individual_vote() {
    let mut deps = setup_poll();

    execute(
        deps.as_mut(),
        env_at(11),
        mock_info("alice", &[]),
        ExecuteMsg::Vote { approve: true },
    )
    .unwrap();

    let bin = query(
        deps.as_ref(),
        env_at(11),
        QueryMsg::Vote {
            voter: "alice".to_string(),
        },
    )
    .unwrap();
    let vote: VoteResponse = cosmwasm_std::from_json(&bin).unwrap();
    assert!(vote.voted);
    assert_eq!(Some(true), vote.approve);

    let bin = query(
        deps.as_ref(),
        env_at(11),
        QueryMsg::Vote {
            voter: "bob".to_string(),
        },
    )
    .unwrap();
    let vote: VoteResponse = cosmwasm_std::from_json(&bin).unwrap();
    assert!(!vote.voted);
    assert_eq!(None, vote.approve);
}
