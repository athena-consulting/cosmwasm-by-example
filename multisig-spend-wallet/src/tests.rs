use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, BankMsg, CosmosMsg};

use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, ProposalResponse, QueryMsg};
use crate::ContractError;

fn setup_wallet() -> cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
> {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        InstantiateMsg {
            owners: vec![
                "owner1".to_string(),
                "owner2".to_string(),
                "owner3".to_string(),
            ],
            threshold: 2,
        },
    )
    .unwrap();
    deps
}

#[test]
fn owner_proposes_and_second_owner_executes_spend() {
    let mut deps = setup_wallet();

    let proposed = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        ExecuteMsg::ProposeSpend {
            recipient: "vendor".to_string(),
            amount: coin(250, "uatom"),
        },
    )
    .unwrap();
    assert_eq!(0, proposed.messages.len());

    let approved = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner2", &[]),
        ExecuteMsg::Approve { proposal_id: 1 },
    )
    .unwrap();

    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "vendor".to_string(),
            amount: vec![coin(250, "uatom")],
        }),
        approved.messages[0].msg
    );

    let bin = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Proposal { proposal_id: 1 },
    )
    .unwrap();
    let proposal: ProposalResponse = cosmwasm_std::from_json(&bin).unwrap();
    assert_eq!(
        vec!["owner1".to_string(), "owner2".to_string()],
        proposal.approvals
    );
    assert!(proposal.executed);
}

#[test]
fn rejects_non_owner_actions_and_duplicate_approvals() {
    let mut deps = setup_wallet();

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("random", &[]),
        ExecuteMsg::ProposeSpend {
            recipient: "vendor".to_string(),
            amount: coin(10, "uatom"),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        ExecuteMsg::ProposeSpend {
            recipient: "vendor".to_string(),
            amount: coin(10, "uatom"),
        },
    )
    .unwrap();

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        ExecuteMsg::Approve { proposal_id: 1 },
    )
    .unwrap_err();
    assert_eq!(ContractError::AlreadyApproved {}, err);

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("random", &[]),
        ExecuteMsg::Approve { proposal_id: 1 },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);
}

#[test]
fn rejects_invalid_configuration_and_amount() {
    let mut deps = mock_dependencies();

    let err = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        InstantiateMsg {
            owners: vec![],
            threshold: 1,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::EmptyOwners {}, err);

    let err = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        InstantiateMsg {
            owners: vec!["owner1".to_string(), "owner1".to_string()],
            threshold: 1,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::DuplicateOwner {}, err);

    let err = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        InstantiateMsg {
            owners: vec!["owner1".to_string()],
            threshold: 2,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::InvalidThreshold {}, err);

    let mut deps = setup_wallet();
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        ExecuteMsg::ProposeSpend {
            recipient: "vendor".to_string(),
            amount: coin(0, "uatom"),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::InvalidAmount {}, err);
}

#[test]
fn threshold_one_executes_on_proposal() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        InstantiateMsg {
            owners: vec!["owner1".to_string()],
            threshold: 1,
        },
    )
    .unwrap();

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner1", &[]),
        ExecuteMsg::ProposeSpend {
            recipient: "vendor".to_string(),
            amount: coin(25, "uatom"),
        },
    )
    .unwrap();

    assert_eq!(1, res.messages.len());
}

#[test]
fn queries_config() {
    let deps = setup_wallet();

    let bin = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = cosmwasm_std::from_json(&bin).unwrap();

    assert_eq!(vec!["owner1", "owner2", "owner3"], config.owners);
    assert_eq!(2, config.threshold);
    assert_eq!(1, config.next_proposal_id);
}
