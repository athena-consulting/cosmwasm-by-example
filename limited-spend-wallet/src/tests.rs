use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, BankMsg, CosmosMsg, Env, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, WindowResponse};
use crate::ContractError;

fn env_at(height: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env
}

fn setup_wallet(
    start_height: u64,
) -> cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
> {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        env_at(start_height),
        mock_info("owner", &[]),
        InstantiateMsg {
            spender: "spender".to_string(),
            denom: "uatom".to_string(),
            limit: Uint128::new(100),
            window_size: 10,
        },
    )
    .unwrap();
    deps
}

#[test]
fn owner_deposits_and_spender_spends_within_limit() {
    let mut deps = setup_wallet(10);

    let deposit = execute(
        deps.as_mut(),
        env_at(10),
        mock_info("owner", &[coin(500, "uatom")]),
        ExecuteMsg::Deposit {},
    )
    .unwrap();
    assert_eq!(0, deposit.messages.len());

    let spend = execute(
        deps.as_mut(),
        env_at(11),
        mock_info("spender", &[]),
        ExecuteMsg::Spend {
            recipient: "vendor".to_string(),
            amount: Uint128::new(75),
        },
    )
    .unwrap();

    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "vendor".to_string(),
            amount: vec![coin(75, "uatom")],
        }),
        spend.messages[0].msg
    );

    let bin = query(deps.as_ref(), env_at(11), QueryMsg::Window {}).unwrap();
    let window: WindowResponse = cosmwasm_std::from_json(&bin).unwrap();
    assert_eq!(Uint128::new(75), window.spent);
    assert_eq!(Uint128::new(25), window.remaining);
}

#[test]
fn rejects_spending_over_limit_until_new_window() {
    let mut deps = setup_wallet(10);

    execute(
        deps.as_mut(),
        env_at(11),
        mock_info("spender", &[]),
        ExecuteMsg::Spend {
            recipient: "vendor".to_string(),
            amount: Uint128::new(70),
        },
    )
    .unwrap();

    let err = execute(
        deps.as_mut(),
        env_at(12),
        mock_info("spender", &[]),
        ExecuteMsg::Spend {
            recipient: "vendor".to_string(),
            amount: Uint128::new(40),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::SpendLimitExceeded {}, err);

    let res = execute(
        deps.as_mut(),
        env_at(20),
        mock_info("spender", &[]),
        ExecuteMsg::Spend {
            recipient: "vendor".to_string(),
            amount: Uint128::new(100),
        },
    )
    .unwrap();

    assert_eq!(1, res.messages.len());
}

#[test]
fn only_spender_can_spend() {
    let mut deps = setup_wallet(10);

    let err = execute(
        deps.as_mut(),
        env_at(11),
        mock_info("random", &[]),
        ExecuteMsg::Spend {
            recipient: "vendor".to_string(),
            amount: Uint128::new(1),
        },
    )
    .unwrap_err();

    assert_eq!(ContractError::Unauthorized {}, err);
}

#[test]
fn owner_updates_spender() {
    let mut deps = setup_wallet(10);

    execute(
        deps.as_mut(),
        env_at(11),
        mock_info("owner", &[]),
        ExecuteMsg::UpdateSpender {
            spender: "new_spender".to_string(),
        },
    )
    .unwrap();

    let err = execute(
        deps.as_mut(),
        env_at(12),
        mock_info("spender", &[]),
        ExecuteMsg::Spend {
            recipient: "vendor".to_string(),
            amount: Uint128::new(1),
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);

    let res = execute(
        deps.as_mut(),
        env_at(12),
        mock_info("new_spender", &[]),
        ExecuteMsg::Spend {
            recipient: "vendor".to_string(),
            amount: Uint128::new(1),
        },
    )
    .unwrap();
    assert_eq!(1, res.messages.len());
}

#[test]
fn validates_initial_config_and_deposits() {
    let mut deps = mock_dependencies();

    let err = instantiate(
        deps.as_mut(),
        env_at(10),
        mock_info("owner", &[]),
        InstantiateMsg {
            spender: "spender".to_string(),
            denom: "uatom".to_string(),
            limit: Uint128::zero(),
            window_size: 10,
        },
    )
    .unwrap_err();
    assert_eq!(ContractError::InvalidLimit {}, err);

    let mut deps = setup_wallet(10);
    let err = execute(
        deps.as_mut(),
        env_at(10),
        mock_info("owner", &[coin(10, "uosmo")]),
        ExecuteMsg::Deposit {},
    )
    .unwrap_err();
    assert_eq!(ContractError::InvalidFunds {}, err);

    let bin = query(deps.as_ref(), env_at(10), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = cosmwasm_std::from_json(&bin).unwrap();
    assert_eq!("owner", config.owner);
    assert_eq!("spender", config.spender);
    assert_eq!("uatom", config.denom);
}
