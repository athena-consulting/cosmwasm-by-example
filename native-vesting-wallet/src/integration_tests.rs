#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, Empty, Timestamp, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StatusResponse};

    const OWNER: &str = "owner";
    const RECIPIENT: &str = "recipient";
    const DENOM: &str = "utoken";
    const START: u64 = 100;
    const CLIFF: u64 = 200;
    const END: u64 = 500;

    fn contract_vesting_wallet() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn mock_app() -> App {
        let mut app = AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(OWNER), coins(2_000, DENOM))
                .unwrap();
        });
        app.update_block(|block| block.time = Timestamp::from_seconds(START));
        app
    }

    fn instantiate_wallet(app: &mut App, initial_amount: u128) -> Addr {
        let code_id = app.store_code(contract_vesting_wallet());
        let msg = InstantiateMsg {
            recipient: RECIPIENT.to_string(),
            denom: DENOM.to_string(),
            start_time: START,
            cliff_time: Some(CLIFF),
            end_time: END,
        };

        app.instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &msg,
            &coins(initial_amount, DENOM),
            "native-vesting-wallet",
            None,
        )
        .unwrap()
    }

    #[test]
    fn instantiate_and_query_config() {
        let mut app = mock_app();
        let contract = instantiate_wallet(&mut app, 1_000);

        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contract, &QueryMsg::Config {})
            .unwrap();

        assert_eq!(config.owner, OWNER);
        assert_eq!(config.recipient, RECIPIENT);
        assert_eq!(config.denom, DENOM);
        assert_eq!(config.start_time, START);
        assert_eq!(config.cliff_time, CLIFF);
        assert_eq!(config.end_time, END);
        assert_eq!(config.total_amount, Uint128::new(1_000));
        assert_eq!(config.released_amount, Uint128::zero());
    }

    #[test]
    fn status_is_locked_before_cliff() {
        let mut app = mock_app();
        let contract = instantiate_wallet(&mut app, 1_000);
        app.update_block(|block| block.time = Timestamp::from_seconds(150));

        let status: StatusResponse = app
            .wrap()
            .query_wasm_smart(contract, &QueryMsg::Status {})
            .unwrap();

        assert_eq!(status.vested_amount, Uint128::zero());
        assert_eq!(status.claimable_amount, Uint128::zero());
        assert_eq!(status.locked_amount, Uint128::new(1_000));
    }

    #[test]
    fn claims_linear_vested_amounts() {
        let mut app = mock_app();
        let contract = instantiate_wallet(&mut app, 1_000);
        app.update_block(|block| block.time = Timestamp::from_seconds(250));

        app.execute_contract(
            Addr::unchecked(RECIPIENT),
            contract.clone(),
            &ExecuteMsg::Claim {},
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_balance(RECIPIENT, DENOM).unwrap().amount,
            Uint128::new(375)
        );

        app.update_block(|block| block.time = Timestamp::from_seconds(END));
        app.execute_contract(
            Addr::unchecked(RECIPIENT),
            contract,
            &ExecuteMsg::Claim {},
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_balance(RECIPIENT, DENOM).unwrap().amount,
            Uint128::new(1_000)
        );
    }

    #[test]
    fn allows_additional_deposits_on_same_schedule() {
        let mut app = mock_app();
        let contract = instantiate_wallet(&mut app, 1_000);

        app.execute_contract(
            Addr::unchecked(OWNER),
            contract.clone(),
            &ExecuteMsg::Deposit {},
            &coins(500, DENOM),
        )
        .unwrap();
        app.update_block(|block| block.time = Timestamp::from_seconds(END));

        let status: StatusResponse = app
            .wrap()
            .query_wasm_smart(contract, &QueryMsg::Status {})
            .unwrap();

        assert_eq!(status.total_amount, Uint128::new(1_500));
        assert_eq!(status.claimable_amount, Uint128::new(1_500));
    }

    #[test]
    fn rejects_invalid_schedule() {
        let mut app = mock_app();
        let code_id = app.store_code(contract_vesting_wallet());
        let msg = InstantiateMsg {
            recipient: RECIPIENT.to_string(),
            denom: DENOM.to_string(),
            start_time: END,
            cliff_time: Some(CLIFF),
            end_time: START,
        };

        let err = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &msg,
                &coins(1_000, DENOM),
                "native-vesting-wallet",
                None,
            )
            .unwrap_err();

        let root = err.root_cause().to_string();
        assert!(
            root.contains("vesting schedule is invalid"),
            "unexpected error: {root}"
        );
    }

    #[test]
    fn rejects_claim_before_vesting() {
        let mut app = mock_app();
        let contract = instantiate_wallet(&mut app, 1_000);
        app.update_block(|block| block.time = Timestamp::from_seconds(150));

        let err = app
            .execute_contract(
                Addr::unchecked(RECIPIENT),
                contract,
                &ExecuteMsg::Claim {},
                &[],
            )
            .unwrap_err();

        let root = err.root_cause().to_string();
        assert!(
            root.contains("no vested funds are claimable"),
            "unexpected error: {root}"
        );
    }
}
