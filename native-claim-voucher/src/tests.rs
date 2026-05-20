use cosmwasm_std::{coin, Addr, Empty};
use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VoucherResponse};

const CREATOR: &str = "creator";
const RECIPIENT: &str = "recipient";
const STRANGER: &str = "stranger";
const DENOM: &str = "uclaim";

fn voucher_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(CREATOR), vec![coin(100, DENOM)])
            .unwrap();
    })
}

fn instantiate_contract(app: &mut App) -> Addr {
    let code_id = app.store_code(voucher_contract());
    app.instantiate_contract(
        code_id,
        Addr::unchecked(CREATOR),
        &InstantiateMsg {},
        &[],
        "native claim voucher",
        None,
    )
    .unwrap()
}

fn create_voucher(app: &mut App, contract: &Addr, id: &str, expires_at_height: u64) {
    app.execute_contract(
        Addr::unchecked(CREATOR),
        contract.clone(),
        &ExecuteMsg::CreateVoucher {
            id: id.to_string(),
            recipient: RECIPIENT.to_string(),
            expires_at_height,
        },
        &[coin(25, DENOM)],
    )
    .unwrap();
}

#[test]
fn recipient_claims_before_expiration() {
    let mut app = mock_app();
    let contract = instantiate_contract(&mut app);
    create_voucher(&mut app, &contract, "invoice-1", 20_000);

    app.execute_contract(
        Addr::unchecked(RECIPIENT),
        contract.clone(),
        &ExecuteMsg::Claim {
            id: "invoice-1".to_string(),
        },
        &[],
    )
    .unwrap();

    let balance = app.wrap().query_balance(RECIPIENT, DENOM).unwrap();
    assert_eq!(balance.amount.u128(), 25);

    let voucher: VoucherResponse = app
        .wrap()
        .query_wasm_smart(
            contract,
            &QueryMsg::Voucher {
                id: "invoice-1".to_string(),
            },
        )
        .unwrap();
    assert_eq!(voucher.status, "claimed");
}

#[test]
fn only_recipient_can_claim() {
    let mut app = mock_app();
    let contract = instantiate_contract(&mut app);
    create_voucher(&mut app, &contract, "invoice-2", 20_000);

    let err = app
        .execute_contract(
            Addr::unchecked(STRANGER),
            contract,
            &ExecuteMsg::Claim {
                id: "invoice-2".to_string(),
            },
            &[],
        )
        .unwrap_err();

    assert!(err
        .root_cause()
        .to_string()
        .contains(&ContractError::Unauthorized {}.to_string()));
}

#[test]
fn creator_refunds_after_expiration() {
    let mut app = mock_app();
    let contract = instantiate_contract(&mut app);
    create_voucher(&mut app, &contract, "invoice-3", 10);

    app.update_block(|block| block.height = 11);
    app.execute_contract(
        Addr::unchecked(CREATOR),
        contract.clone(),
        &ExecuteMsg::RefundExpired {
            id: "invoice-3".to_string(),
        },
        &[],
    )
    .unwrap();

    let balance = app.wrap().query_balance(CREATOR, DENOM).unwrap();
    assert_eq!(balance.amount.u128(), 100);

    let voucher: VoucherResponse = app
        .wrap()
        .query_wasm_smart(
            contract,
            &QueryMsg::Voucher {
                id: "invoice-3".to_string(),
            },
        )
        .unwrap();
    assert_eq!(voucher.status, "refunded");
}
