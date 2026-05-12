#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ConfigResponse, ExecuteMsg, InstantiateMsg, MemberInput, PreviewResponse, QueryMsg,
    };

    const SENDER: &str = "sender";
    const ALICE: &str = "alice";
    const BOB: &str = "bob";
    const CAROL: &str = "carol";
    const DENOM: &str = "utoken";

    fn contract_payment_splitter() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(SENDER), coins(2_000, DENOM))
                .unwrap();
        })
    }

    fn instantiate_splitter(app: &mut App, members: Vec<MemberInput>) -> Addr {
        let code_id = app.store_code(contract_payment_splitter());
        let msg = InstantiateMsg {
            denom: DENOM.to_string(),
            members,
            remainder_recipient: Some(SENDER.to_string()),
        };

        app.instantiate_contract(
            code_id,
            Addr::unchecked(SENDER),
            &msg,
            &[],
            "payment-splitter",
            None,
        )
        .unwrap()
    }

    fn member(address: &str, share: u64) -> MemberInput {
        MemberInput {
            address: address.to_string(),
            share,
        }
    }

    #[test]
    fn instantiate_and_query_config() {
        let mut app = mock_app();
        let contract = instantiate_splitter(
            &mut app,
            vec![member(ALICE, 2), member(BOB, 1), member(CAROL, 1)],
        );

        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contract, &QueryMsg::Config {})
            .unwrap();

        assert_eq!(config.denom, DENOM);
        assert_eq!(config.total_shares, 4);
        assert_eq!(config.remainder_recipient, SENDER);
        assert_eq!(config.members.len(), 3);
        assert_eq!(config.members[0].address, ALICE);
        assert_eq!(config.members[0].share, 2);
    }

    #[test]
    fn distributes_native_tokens_by_share() {
        let mut app = mock_app();
        let contract = instantiate_splitter(
            &mut app,
            vec![member(ALICE, 2), member(BOB, 1), member(CAROL, 1)],
        );

        app.execute_contract(
            Addr::unchecked(SENDER),
            contract,
            &ExecuteMsg::Distribute {},
            &coins(1_000, DENOM),
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_balance(ALICE, DENOM).unwrap().amount,
            Uint128::new(500)
        );
        assert_eq!(
            app.wrap().query_balance(BOB, DENOM).unwrap().amount,
            Uint128::new(250)
        );
        assert_eq!(
            app.wrap().query_balance(CAROL, DENOM).unwrap().amount,
            Uint128::new(250)
        );
    }

    #[test]
    fn returns_remainder_to_remainder_recipient() {
        let mut app = mock_app();
        let contract = instantiate_splitter(&mut app, vec![member(ALICE, 2), member(BOB, 3)]);

        app.execute_contract(
            Addr::unchecked(SENDER),
            contract,
            &ExecuteMsg::Distribute {},
            &coins(7, DENOM),
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_balance(ALICE, DENOM).unwrap().amount,
            Uint128::new(2)
        );
        assert_eq!(
            app.wrap().query_balance(BOB, DENOM).unwrap().amount,
            Uint128::new(4)
        );
        assert_eq!(
            app.wrap().query_balance(SENDER, DENOM).unwrap().amount,
            Uint128::new(1_994)
        );
    }

    #[test]
    fn previews_a_split_without_sending_tokens() {
        let mut app = mock_app();
        let contract = instantiate_splitter(&mut app, vec![member(ALICE, 2), member(BOB, 3)]);

        let preview: PreviewResponse = app
            .wrap()
            .query_wasm_smart(
                contract,
                &QueryMsg::Preview {
                    amount: Uint128::new(7),
                },
            )
            .unwrap();

        assert_eq!(preview.shares[0].amount, Uint128::new(2));
        assert_eq!(preview.shares[1].amount, Uint128::new(4));
        assert_eq!(preview.remainder, Uint128::new(1));
        assert_eq!(preview.remainder_recipient, SENDER);
    }

    #[test]
    fn rejects_duplicate_members() {
        let mut app = mock_app();
        let code_id = app.store_code(contract_payment_splitter());
        let msg = InstantiateMsg {
            denom: DENOM.to_string(),
            members: vec![member(ALICE, 1), member(ALICE, 1)],
            remainder_recipient: None,
        };

        let err = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(SENDER),
                &msg,
                &[],
                "payment-splitter",
                None,
            )
            .unwrap_err();

        let root = err.root_cause().to_string();
        assert!(
            root.contains("duplicate member address"),
            "unexpected error: {root}"
        );
    }

    #[test]
    fn rejects_distribute_without_matching_funds() {
        let mut app = mock_app();
        let contract = instantiate_splitter(&mut app, vec![member(ALICE, 1), member(BOB, 1)]);

        let err = app
            .execute_contract(
                Addr::unchecked(SENDER),
                contract,
                &ExecuteMsg::Distribute {},
                &[],
            )
            .unwrap_err();

        let root = err.root_cause().to_string();
        assert!(
            root.contains("no funds were sent"),
            "unexpected error: {root}"
        );
    }
}
