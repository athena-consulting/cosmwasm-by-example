use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, BankMsg, CosmosMsg, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RecipientShareMsg};
use crate::ContractError;

fn recipient(address: &str, share: u128) -> RecipientShareMsg {
    RecipientShareMsg {
        address: address.to_string(),
        share: Uint128::new(share),
    }
}

fn instantiate_splitter() -> cosmwasm_std::OwnedDeps<
    cosmwasm_std::MemoryStorage,
    cosmwasm_std::testing::MockApi,
    cosmwasm_std::testing::MockQuerier,
> {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        recipients: vec![recipient("alice", 1), recipient("bob", 3)],
    };
    instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();
    deps
}

#[test]
fn distributes_native_funds_by_configured_shares() {
    let mut deps = instantiate_splitter();
    let info = mock_info("payer", &[coin(100, "uatom")]);

    let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Distribute {}).unwrap();

    assert_eq!(2, res.messages.len());
    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "alice".to_string(),
            amount: vec![coin(25, "uatom")],
        }),
        res.messages[0].msg
    );
    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "bob".to_string(),
            amount: vec![coin(75, "uatom")],
        }),
        res.messages[1].msg
    );
}

#[test]
fn sends_integer_remainder_to_first_recipient() {
    let mut deps = instantiate_splitter();
    let info = mock_info("payer", &[coin(101, "uatom")]);

    let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Distribute {}).unwrap();

    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "alice".to_string(),
            amount: vec![coin(26, "uatom")],
        }),
        res.messages[0].msg
    );
    assert_eq!(
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "bob".to_string(),
            amount: vec![coin(75, "uatom")],
        }),
        res.messages[1].msg
    );
}

#[test]
fn rejects_distribution_without_exactly_one_native_coin() {
    let mut deps = instantiate_splitter();

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("payer", &[]),
        ExecuteMsg::Distribute {},
    )
    .unwrap_err();

    assert_eq!(ContractError::InvalidFunds {}, err);
}

#[test]
fn rejects_invalid_recipient_configuration() {
    let mut deps = mock_dependencies();
    let no_recipients = InstantiateMsg { recipients: vec![] };
    let err = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("creator", &[]),
        no_recipients,
    )
    .unwrap_err();
    assert_eq!(ContractError::NoRecipients {}, err);

    let zero_share = InstantiateMsg {
        recipients: vec![recipient("alice", 0)],
    };
    let err = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("creator", &[]),
        zero_share,
    )
    .unwrap_err();
    assert_eq!(ContractError::ZeroShare {}, err);
}

#[test]
fn queries_payment_splitter_config() {
    let deps = instantiate_splitter();

    let bin = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let response: ConfigResponse = cosmwasm_std::from_json(&bin).unwrap();

    assert_eq!(Uint128::new(4), response.total_shares);
    assert_eq!(recipient("alice", 1), response.recipients[0]);
    assert_eq!(recipient("bob", 3), response.recipients[1]);
}
