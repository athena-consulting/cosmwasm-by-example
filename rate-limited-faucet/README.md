# Rate Limited Faucet

The rate limited faucet is a simple native-token faucet for testnets, workshops, or local dev chains. It keeps a per-address claim timestamp so each wallet can request a fixed amount only once per cooldown window.

## Why use a cooldown?

A faucet usually holds a shared pool of tokens. Without limits, one address can drain that pool in a few transactions. This example shows how to combine:

- native bank transfers with `BankMsg::Send`
- owner-managed configuration
- per-address state with `Map`
- read queries that expose the next available claim time

## Instantiate

```rust
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub claim_amount: Uint128,
    pub cooldown_seconds: u64,
}
```

If `owner` is omitted, the sender that instantiates the contract becomes the owner.

## Execute

```rust
pub enum ExecuteMsg {
    Claim {},
    Fund {},
    UpdateConfig {
        owner: Option<String>,
        claim_amount: Option<Uint128>,
        cooldown_seconds: Option<u64>,
    },
}
```

- `Claim` sends `claim_amount` of `denom` to the caller when their cooldown has expired.
- `Fund` is a convenience entry point for sending native tokens into the contract.
- `UpdateConfig` can only be called by the owner.

## Query

```rust
pub enum QueryMsg {
    Config {},
    LastClaim { address: String },
    NextClaim { address: String },
}
```

`NextClaim` returns the timestamp when an address may claim again and a `can_claim` boolean for UI use.

## Testing

Run the contract tests from this directory:

```bash
cargo test
```
