# Native Milestone Escrow

This example shows a native-token escrow for milestone payments. A buyer funds a
milestone, a seller completes the work, and either the buyer or an arbiter can
release funds to the seller. If the milestone fails, the arbiter or seller can
refund the buyer, and the buyer can self-refund after an optional timeout.

It demonstrates:

- creating funded escrow records with native tokens
- validating exact native-token payments
- using optional per-escrow or default arbiters
- release and refund settlement flows
- timeout-based buyer refunds
- querying escrow and contract configuration state

## Flow

1. Instantiate with a native denom and optional default arbiter.
2. A buyer creates an escrow with the exact native token amount, seller, arbiter,
   description, and optional timeout.
3. The buyer or arbiter releases the escrow to the seller when the milestone is
   complete.
4. The arbiter or seller can refund the buyer. The buyer can refund themselves
   after the escrow timeout.
5. Settled escrows cannot be released or refunded again.

## Messages

### Instantiate

```rust
pub struct InstantiateMsg {
    pub denom: String,
    pub owner: Option<String>,
    pub default_arbiter: Option<String>,
}
```

### Execute

```rust
pub enum ExecuteMsg {
    CreateEscrow {
        id: Option<u64>,
        seller: String,
        arbiter: Option<String>,
        amount: Uint128,
        description: String,
        timeout: Option<u64>,
    },
    Release {
        id: u64,
    },
    Refund {
        id: u64,
    },
    UpdateConfig {
        owner: Option<String>,
        default_arbiter: Option<String>,
    },
}
```

### Query

```rust
pub enum QueryMsg {
    Config {},
    Escrow { id: u64 },
    NextEscrowId {},
}
```
