# Native Subscription

This example shows a simple subscription contract that accepts one native token
denomination and grants time-based access to each subscriber.

It demonstrates:

- Validating native-token payments.
- Extending access from either the current block time or an existing expiry.
- Querying whether an address currently has access.
- Letting the admin update plan settings and withdraw collected funds.

## Flow

1. Instantiate the contract with a token denom, a price, and a subscription
   duration in seconds.
2. A user calls `subscribe` and attaches at least the configured price in the
   configured denom.
3. The contract stores the user's `expires_at` timestamp.
4. Anyone can query `subscription` or `is_active` for a subscriber.
5. The admin can withdraw collected funds or update price, duration, or admin.

## Messages

```rust
InstantiateMsg {
    denom: "uatom".to_string(),
    price: Uint128::new(1_000_000),
    period_seconds: 2_592_000,
    admin: None,
}
```

```rust
ExecuteMsg::Subscribe {}
ExecuteMsg::Withdraw {
    recipient: None,
    amount: None,
}
ExecuteMsg::UpdateConfig {
    price: Some(Uint128::new(2_000_000)),
    period_seconds: None,
    admin: None,
}
```

```rust
QueryMsg::Config {}
QueryMsg::Subscription {
    subscriber: "cosmos1...".to_string(),
}
QueryMsg::IsActive {
    subscriber: "cosmos1...".to_string(),
}
QueryMsg::Treasury {}
```

## Tests

Run:

```bash
cargo test
```
