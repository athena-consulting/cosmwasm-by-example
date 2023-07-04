#![cfg(test)]

use std::borrow::BorrowMut;

use crate::error::ContractError;
use cosmwasm_std::{coins, Addr, Coin, Decimal, Empty, Uint128};
use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, Denom};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use std::str::FromStr;

use crate::msg::{ExecuteMsg, FeeResponse, InfoResponse, InstantiateMsg, QueryMsg, TokenSelect};

fn mock_app() -> App {
    App::default()
}

pub fn contract_amm() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

fn get_info(router: &App, contract_addr: &Addr) -> InfoResponse {
    router
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Info {})
        .unwrap()
}

fn get_fee(router: &App, contract_addr: &Addr) -> FeeResponse {
    router
        .wrap()
        .query_wasm_smart(contract_addr, &QueryMsg::Fee {})
        .unwrap()
}

fn create_amm(
    router: &mut App,
    owner: &Addr,
    token1_denom: Denom,
    token2_denom: Denom,
    lp_fee_percent: Decimal,
    protocol_fee_percent: Decimal,
    protocol_fee_recipient: String,
) -> Addr {
    // set up amm contract
    let cw20_id = router.store_code(contract_cw20());
    let amm_id = router.store_code(contract_amm());
    let msg = InstantiateMsg {
        token1_denom,
        token2_denom,
        lp_token_code_id: cw20_id,
        owner: Some(owner.to_string()),
        lp_fee_percent,
        protocol_fee_percent,
        protocol_fee_recipient,
    };
    router
        .instantiate_contract(amm_id, owner.clone(), &msg, &[], "amm", None)
        .unwrap()
}

// CreateCW20 create new cw20 with given initial balance belonging to owner
fn create_cw20(
    router: &mut App,
    owner: &Addr,
    name: String,
    symbol: String,
    balance: Uint128,
) -> Cw20Contract {
    // set up cw20 contract with some tokens
    let cw20_id = router.store_code(contract_cw20());
    let msg = cw20_base::msg::InstantiateMsg {
        name,
        symbol,
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: owner.to_string(),
            amount: balance,
        }],
        mint: None,
        marketing: None,
    };
    let addr = router
        .instantiate_contract(cw20_id, owner.clone(), &msg, &[], "CASH", None)
        .unwrap();
    Cw20Contract(addr)
}

fn bank_balance(router: &mut App, addr: &Addr, denom: String) -> Coin {
    router
        .wrap()
        .query_balance(addr.to_string(), denom)
        .unwrap()
}

#[test]
// receive cw20 tokens and release upon approval
fn test_instantiate() {
    let mut router = mock_app();

    const NATIVE_TOKEN_DENOM: &str = "juno";

    let owner = Addr::unchecked("owner");
    let funds = coins(2000, NATIVE_TOKEN_DENOM);
    router.borrow_mut().init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &owner, funds).unwrap()
    });

    let cw20_token = create_cw20(
        &mut router,
        &owner,
        "token".to_string(),
        "CWTOKEN".to_string(),
        Uint128::new(5000),
    );

    let lp_fee_percent = Decimal::from_str("0.3").unwrap();
    let protocol_fee_percent = Decimal::zero();
    let amm_addr = create_amm(
        &mut router,
        &owner,
        Denom::Native(NATIVE_TOKEN_DENOM.into()),
        Denom::Cw20(cw20_token.addr()),
        lp_fee_percent,
        protocol_fee_percent,
        owner.to_string(),
    );

    assert_ne!(cw20_token.addr(), amm_addr);

    let _info = get_info(&router, &amm_addr);

    let fee = get_fee(&router, &amm_addr);
    assert_eq!(fee.lp_fee_percent, lp_fee_percent);
    assert_eq!(fee.protocol_fee_percent, protocol_fee_percent);
    assert_eq!(fee.protocol_fee_recipient, owner.to_string());
    assert_eq!(fee.owner.unwrap(), owner.to_string());

    // Test instantiation with invalid fee amount
    let lp_fee_percent = Decimal::from_str("1.01").unwrap();
    let protocol_fee_percent = Decimal::zero();
    let cw20_id = router.store_code(contract_cw20());
    let amm_id = router.store_code(contract_amm());
    let msg = InstantiateMsg {
        token1_denom: Denom::Native(NATIVE_TOKEN_DENOM.into()),
        token2_denom: Denom::Cw20(cw20_token.addr()),
        lp_token_code_id: cw20_id,
        owner: Some(owner.to_string()),
        lp_fee_percent,
        protocol_fee_percent,
        protocol_fee_recipient: owner.to_string(),
    };
    let err = router
        .instantiate_contract(amm_id, owner.clone(), &msg, &[], "amm", None)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        ContractError::FeesTooHigh {
            max_fee_percent: Decimal::from_str("1").unwrap(),
            total_fee_percent: Decimal::from_str("1.01").unwrap()
        },
        err
    );
}


#[test]
fn update_config() {
    let mut router = mock_app();

    const NATIVE_TOKEN_DENOM: &str = "juno";

    let owner = Addr::unchecked("owner");
    let funds = coins(2000, NATIVE_TOKEN_DENOM);
    router.borrow_mut().init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &owner, funds).unwrap()
    });

    let cw20_token = create_cw20(
        &mut router,
        &owner,
        "token".to_string(),
        "CWTOKEN".to_string(),
        Uint128::new(5000),
    );

    let lp_fee_percent = Decimal::from_str("0.3").unwrap();
    let protocol_fee_percent = Decimal::zero();
    let amm_addr = create_amm(
        &mut router,
        &owner,
        Denom::Native(NATIVE_TOKEN_DENOM.to_string()),
        Denom::Cw20(cw20_token.addr()),
        lp_fee_percent,
        protocol_fee_percent,
        owner.to_string(),
    );

    let lp_fee_percent = Decimal::from_str("0.15").unwrap();
    let protocol_fee_percent = Decimal::from_str("0.15").unwrap();
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some(owner.to_string()),
        protocol_fee_recipient: "new_fee_recpient".to_string(),
        lp_fee_percent,
        protocol_fee_percent,
    };
    let _res = router
        .execute_contract(owner.clone(), amm_addr.clone(), &msg, &[])
        .unwrap();

    let fee = get_fee(&router, &amm_addr);
    assert_eq!(fee.protocol_fee_recipient, "new_fee_recpient".to_string());
    assert_eq!(fee.protocol_fee_percent, protocol_fee_percent);
    assert_eq!(fee.lp_fee_percent, lp_fee_percent);
    assert_eq!(fee.owner.unwrap(), owner.to_string());

    // Try updating config with fee values that are too high
    let lp_fee_percent = Decimal::from_str("1.01").unwrap();
    let protocol_fee_percent = Decimal::zero();
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some(owner.to_string()),
        protocol_fee_recipient: "new_fee_recpient".to_string(),
        lp_fee_percent,
        protocol_fee_percent,
    };
    let err = router
        .execute_contract(owner.clone(), amm_addr.clone(), &msg, &[])
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        ContractError::FeesTooHigh {
            max_fee_percent: Decimal::from_str("1").unwrap(),
            total_fee_percent: Decimal::from_str("1.01").unwrap()
        },
        err
    );

    // Try updating config with invalid owner, show throw unauthoritzed error
    let lp_fee_percent = Decimal::from_str("0.21").unwrap();
    let protocol_fee_percent = Decimal::from_str("0.09").unwrap();
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some(owner.to_string()),
        protocol_fee_recipient: owner.to_string(),
        lp_fee_percent,
        protocol_fee_percent,
    };
    let err = router
        .execute_contract(
            Addr::unchecked("invalid_owner"),
            amm_addr.clone(),
            &msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(ContractError::Unauthorized {}, err);

    // Try updating owner and fee params
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("new_owner".to_string()),
        protocol_fee_recipient: owner.to_string(),
        lp_fee_percent,
        protocol_fee_percent,
    };
    let _res = router
        .execute_contract(owner.clone(), amm_addr.clone(), &msg, &[])
        .unwrap();

    let fee = get_fee(&router, &amm_addr);
    assert_eq!(fee.protocol_fee_recipient, owner.to_string());
    assert_eq!(fee.protocol_fee_percent, protocol_fee_percent);
    assert_eq!(fee.lp_fee_percent, lp_fee_percent);
    assert_eq!(fee.owner.unwrap(), "new_owner".to_string());
}

#[test]
fn test_pass_through_swap() {
    let mut router = mock_app();

    const NATIVE_TOKEN_DENOM: &str = "juno";

    let owner = Addr::unchecked("owner");
    let funds = coins(2000, NATIVE_TOKEN_DENOM);
    router.borrow_mut().init_modules(|router, _, storage| {
        router.bank.init_balance(storage, &owner, funds).unwrap()
    });

    let token1 = create_cw20(
        &mut router,
        &owner,
        "token1".to_string(),
        "TOKENONE".to_string(),
        Uint128::new(5000),
    );
    let token2 = create_cw20(
        &mut router,
        &owner,
        "token2".to_string(),
        "TOKENTWO".to_string(),
        Uint128::new(5000),
    );

    let lp_fee_percent = Decimal::from_str("0.3").unwrap();
    let protocol_fee_percent = Decimal::zero();
    let amm = create_amm(
        &mut router,
        &owner,
        Denom::Cw20(token1.addr()),
        Denom::Cw20(token2.addr()),
        lp_fee_percent,
        protocol_fee_percent,
        owner.to_string(),
    );

    // Add initial liquidity to both pools
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: amm.to_string(),
        amount: Uint128::new(100),
        expires: None,
    };
    let _res = router
        .execute_contract(owner.clone(), token1.addr(), &allowance_msg, &[])
        .unwrap();
    let _res = router
        .execute_contract(owner.clone(), token2.addr(), &allowance_msg, &[])
        .unwrap();

    let add_liquidity_msg = ExecuteMsg::AddLiquidity {
        token1_amount: Uint128::new(100),
        min_liquidity: Uint128::new(100),
        token2_amount: Uint128::new(100),
        expiration: None,
    };
    router
        .execute_contract(
            owner.clone(),
            amm.clone(),
            &add_liquidity_msg,
            &[Coin {
                denom: NATIVE_TOKEN_DENOM.into(),
                amount: Uint128::zero(),
            }],
        )
        .unwrap();

    // Swap token1 for token2
    let allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: amm.to_string(),
        amount: Uint128::new(10),
        expires: None,
    };
    let _res = router
        .execute_contract(owner.clone(), token1.addr(), &allowance_msg, &[])
        .unwrap();

    let swap_msg = ExecuteMsg::Swap {
        input_token: TokenSelect::Token1,
        input_amount: Uint128::new(10),
        min_output: Uint128::new(8),
        expiration: None,
    };
    let _res = router
        .execute_contract(owner.clone(), amm.clone(), &swap_msg, &[])
        .unwrap();

    // ensure balances updated
    let token1_balance = token1.balance(&router, owner.clone()).unwrap();
    assert_eq!(token1_balance, Uint128::new(4890));

    let token2_balance = token2.balance(&router, owner.clone()).unwrap();
    assert_eq!(token2_balance, Uint128::new(4909));

    let amm_native_balance = bank_balance(&mut router, &amm, NATIVE_TOKEN_DENOM.to_string());
    assert_eq!(amm_native_balance.amount, Uint128::zero());

    // assert internal state is consistent
    let info_amm: InfoResponse = get_info(&router, &amm);
    println!("{:?}", info_amm);
    let token1_balance = token1.balance(&router, amm.clone()).unwrap();
    let token2_balance = token2.balance(&router, amm.clone()).unwrap();
    println!("{} {}", token1_balance, token2_balance);
    assert_eq!(info_amm.token2_reserve, token2_balance);
    assert_eq!(info_amm.token1_reserve, token1_balance);
}
