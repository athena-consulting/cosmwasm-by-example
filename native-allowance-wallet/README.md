# Native Allowance Wallet

This example shows a simple native-token allowance wallet. An owner funds the
contract and grants a spender permission to withdraw up to a configured amount
during each time period.

It demonstrates:

- storing owner and spender configuration
- accepting native-token deposits
- enforcing a per-period withdrawal limit
- rolling the allowance period forward from block time
- sending native funds to a recipient selected by the spender
- querying allowance state and contract balance

## Flow

1. Instantiate with a native denom, spender, period length, and period limit.
2. The owner or any supporter deposits the configured native denom.
3. The spender withdraws to themselves or another recipient while staying under
   the current period limit.
4. When block time moves into a later period, the spent amount resets.
5. The owner can update the owner, spender, period limit, or period length.

## Messages

### Instantiate

```rust
pub struct InstantiateMsg {
    pub denom: String,
    pub owner: Option<String>,
    pub spender: String,
    pub period_seconds: u64,
    pub period_limit: Uint128,
}
```

### Execute

```rust
pub enum ExecuteMsg {
    Deposit {},
    Withdraw {
        amount: Uint128,
        recipient: Option<String>,
    },
    UpdateConfig {
        owner: Option<String>,
        spender: Option<String>,
        period_seconds: Option<u64>,
        period_limit: Option<Uint128>,
        reset_period: bool,
    },
}
```

### Query

```rust
pub enum QueryMsg {
    Config {},
    Allowance {},
}
```
