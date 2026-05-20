use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, BankMsg, CosmosMsg, Env, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VestingResponse};
use crate::ContractError;

fn env_at(height: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env
}

fn setup_vesting() -> cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
> {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        env_at(100),
        mock_info("creator", &[coin(1_000, "uatom")]),
        InstantiateMsg {
            beneficiary: "beneficiary".to_string(),
            start_height: 100,
            cliff_height: 120,
            end_height: 200,
        },
    )
    .unwrap();
    deps
}

#[test]
fn rejects_invalid_funding_and_schedule() {
    let mut deps = mock_dependencies();

    let err = instantiate(
        deps.as_mut(),
        env_at(100),
        mock_info("creator", &[]),
        InstantiateMsg {
            beneficiary: "beneficiary".to_string(),
            start_height: 100,
            cliff_height: 120,
            end_height: 200,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::InvalidFunds {}, err);

    let err = instantiate(
        deps.as_mut(),
        env_at(100),
        mock_info("creator", &[coin(1_000, "uatom")]),
        InstantiateMsg {
            beneficiary: "beneficiary".to_string(),
            start_height: 100,
            cliff_height: 200,
            end_height: 200,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::InvalidSchedule {}, err);
}

#[test]
fn reports_no_claimable_amount_before_cliff() {
    let deps = setup_vesting();

    let bin = query(deps.as_ref(), env_at(119), QueryMsg::Vesting {}).unwrap();
    let vesting: VestingResponse = cosmwasm_std::from_json(&bin).unwrap();

    assert_eq!("creator", vesting.creator);
    assert_eq!("beneficiary", vesting.beneficiary);
    assert_eq!("uatom", vesting.denom);
    assert_eq!(Uint128::new(1_000), vesting.total);
    assert_eq!(Uint128::zero(), vesting.vested);
    assert_eq!(Uint128::zero(), vesting.claimable);
}

#[test]
fn beneficiary_claims_linearly_vested_funds() {
    let mut deps = setup_vesting();

    let res = execute(
        deps.as_mut(),
        env_at(150),
        mock_info("beneficiary", &[]),
        ExecuteMsg::Claim {},
    )
    .unwrap();

    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "beneficiary".to_string(),
            amount: vec![coin(500, "uatom")],
        }),
        res.messages[0].msg
    );

    let bin = query(deps.as_ref(), env_at(150), QueryMsg::Vesting {}).unwrap();
    let vesting: VestingResponse = cosmwasm_std::from_json(&bin).unwrap();
    assert_eq!(Uint128::new(500), vesting.claimed);
    assert_eq!(Uint128::zero(), vesting.claimable);
}

#[test]
fn only_beneficiary_can_claim_and_claim_requires_vested_amount() {
    let mut deps = setup_vesting();

    let err = execute(
        deps.as_mut(),
        env_at(150),
        mock_info("random", &[]),
        ExecuteMsg::Claim {},
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    let err = execute(
        deps.as_mut(),
        env_at(119),
        mock_info("beneficiary", &[]),
        ExecuteMsg::Claim {},
    )
    .unwrap_err();
    assert_eq!(ContractError::NothingToClaim {}, err);
}

#[test]
fn final_claim_releases_remaining_funds() {
    let mut deps = setup_vesting();

    execute(
        deps.as_mut(),
        env_at(150),
        mock_info("beneficiary", &[]),
        ExecuteMsg::Claim {},
    )
    .unwrap();

    let res = execute(
        deps.as_mut(),
        env_at(200),
        mock_info("beneficiary", &[]),
        ExecuteMsg::Claim {},
    )
    .unwrap();

    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "beneficiary".to_string(),
            amount: vec![coin(500, "uatom")],
        }),
        res.messages[0].msg
    );

    let bin = query(deps.as_ref(), env_at(250), QueryMsg::Vesting {}).unwrap();
    let vesting: VestingResponse = cosmwasm_std::from_json(&bin).unwrap();
    assert_eq!(Uint128::new(1_000), vesting.claimed);
    assert_eq!(Uint128::zero(), vesting.claimable);
}
