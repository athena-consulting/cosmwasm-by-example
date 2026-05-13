# Native Threshold Vault

This example shows a native-token vault controlled by a fixed member set and a
signature threshold. Members can deposit funds, propose a spend, approve it, and
execute the spend once enough members have approved.

It is useful for learning how to:

- validate and store a member set
- enforce threshold-based authorization
- model spend proposals in contract state
- prevent duplicate approvals and repeated execution
- query a contract's native-token balance before sending funds

## Flow

1. Instantiate the contract with members, a threshold, and a native denom.
2. Anyone can fund the vault with `Deposit {}` by sending the configured denom.
3. A member creates a spend proposal with `ProposeSpend`.
4. Other members approve with `Approve`.
5. Once approvals reach the threshold, a member executes the proposal and the
   vault sends native tokens to the recipient.

## Messages

```rust
pub enum ExecuteMsg {
    Deposit {},
    ProposeSpend {
        recipient: String,
        amount: Uint128,
        description: String,
    },
    Approve {
        proposal_id: u64,
    },
    Execute {
        proposal_id: u64,
    },
}
```

## Queries

- `Config {}` returns the member list, threshold, denom, and next proposal id.
- `Proposal { proposal_id }` returns the proposal, approvals, and execution
  status.

## Tests

Run the tests from this directory:

```bash
cargo test
```

The tests cover invalid thresholds, deposits, member-only proposals, duplicate
approval rejection, threshold execution, insufficient funds, and repeated
execution prevention.
