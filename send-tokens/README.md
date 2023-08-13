# Send Tokens
Smart Contract that can send a blockchain's native tokens to a user specified by the sender in the execute message.

## send_tokens function
After the function use case is completed (in that case empty), we add a bank message for the contract to execute. This means that the contract is the signer of the transaction and not the original sender.
```rust
    // contract.rs 

    pub fn send_tokens(_deps: DepsMut, amount: Uint128, denom: String, to: Addr) -> Result<Response, ContractError> {

        Ok(Response::new().add_attribute("action", "send")
        /* Sending tokens is part of the response of a function
        Developer creates a BankMsg to send tokens to an address with a specific native token
        Will fail if smart contract does not have this much tokens initially.
        If a function raises an error before reaching the response, then no funds are sent.  */
        .add_message(BankMsg::Send { to_address: to.into_string(), amount: vec![Coin{denom, amount}] }))
    }
```

## Integration Testing
Integration testing will be explained in details in another example but it is important to note that to replicate a local blockchain it is importnat to use integration testing as unit testing is aimed to test the functionality of one single function. 

```rust
// integration_tests.rs

 fn balance() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let msg = ExecuteMsg::SendTokens { amount: Uint128::new(10), denom: "token".to_string(), to: Addr::unchecked("receiver") } ;
            let funds_sent = Coin::new(10u128, "token".to_string());
            let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
            app.execute(Addr::unchecked(USER), cosmos_msg).unwrap(); 
            let balance = app.wrap().query_balance("receiver", "token").unwrap();
            assert_eq!(balance.amount, Uint128::new(10));
            assert_eq!(balance.denom, "token");
            
        }
```