# Native Deposit Refund

This example shows a small native-token deposit contract. A user locks an exact
deposit amount in the contract, can reclaim it before a deadline, and the admin
can forfeit a still-locked deposit after the deadline.

It is useful for learning how to:

- validate exact native-token payments with `MessageInfo::funds`
- store one deposit per account with `cw-storage-plus::Map`
- send native tokens back with `BankMsg::Send`
- enforce a deadline with `Env::block.time`
- separate user refunds from admin-only forfeits

## Flow

1. The contract is instantiated with an admin, denom, deposit amount, and refund
   deadline.
2. A user calls `Deposit {}` and sends exactly the configured amount of the
   configured denom.
3. Before the deadline, the user can call `Refund {}` to receive the deposit
   back.
4. After the deadline, the admin can call `Forfeit { depositor, recipient }` to
   send an unreclaimed deposit to a recipient account.

## Messages

```rust
pub enum ExecuteMsg {
    Deposit {},
    Refund {},
    Forfeit {
        depositor: String,
        recipient: String,
    },
}
```

## Queries

- `Config {}` returns the admin, denom, required deposit amount, and deadline.
- `Deposit { address }` returns whether the address currently has a locked
  deposit.

## Tests

Run the example tests from this directory:

```bash
cargo test
```

The tests cover exact-fund validation, duplicate deposits, user refunds,
admin-only forfeits, and the deadline checks.
