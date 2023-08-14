# **Send Tokens**

This document outlines a smart contract designed to send a blockchain's native tokens to a recipient specified by the original sender in the execute message.

## **1. `send_tokens` Function**

Once the main use case of this function is executed (which in this context is void), a bank message is appended for the contract to act upon. It's worth noting that the contract becomes the signer of the transaction, not the initiating sender.

```rust
// contract.rs

pub fn send_tokens(
    _deps: DepsMut,
    amount: Uint128,
    denom: String,
    to: Addr
) -> Result<Response, ContractError> {
    
    /* Sending tokens is managed via the response of this function.
       A developer crafts a BankMsg to transmit tokens to a specified address using the native token.
       The function will fail if the smart contract lacks sufficient tokens.
       If any error surfaces prior to the response's generation, funds won't be transmitted. */
    
    Ok(Response::new()
        .add_attribute("action", "send")
        .add_message(BankMsg::Send {
            to_address: to.into_string(),
            amount: vec![Coin{denom, amount}]
        })
    )
}
```

## **2. Integration Testing**

```rust
// integration_tests.rs

fn balance() {
    let (mut app, cw_template_contract) = proper_instantiate();

    let msg = ExecuteMsg::SendTokens {
        amount: Uint128::new(10),
        denom: "token".to_string(),
        to: Addr::unchecked("receiver")
    };

    let funds_sent = Coin::new(10u128, "token".to_string());
    let cosmos_msg = cw_template_contract.call(msg, funds_sent).unwrap();
    app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();

    let balance = app.wrap().query_balance("receiver", "token").unwrap();
    assert_eq!(balance.amount, Uint128::new(10));
    assert_eq!(balance.denom, "token");
}
```

