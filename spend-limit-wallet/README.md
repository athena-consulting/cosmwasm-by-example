# Spend Limit Wallet

This example shows how to build a shared native-token wallet with owner-managed
spending limits. The owner can give an address permission to spend up to a fixed
amount of one denom during a block-height period. When the period expires, the
spent amount resets automatically.

The contract is intentionally small, but it demonstrates useful CosmWasm
patterns:

- owner-only configuration changes
- validating and storing addresses
- tracking per-address allowances
- resetting period-based state from `env.block.height`
- sending native tokens with `BankMsg::Send`
- querying remaining spendable allowance

## Instantiate

```json
{
  "owner": null,
  "denom": "uatom",
  "default_period": 1000
}
```
If `owner` is `null`, the sender becomes the wallet owner. The `default_period`
is measured in block heights.

## Execute

Set an allowance for a spender:

```json
{
  "set_allowance": {
    "spender": "cosmos1spender...",
    "limit": "250000",
    "period": null
  }
}
```

Spend from the wallet:

```json
{
  "spend": {
    "recipient": "cosmos1recipient...",
    "amount": "100000"
  }
}
```

Remove an allowance:

```json
{
  "remove_allowance": {
    "spender": "cosmos1spender..."
  }
}
```

Update the owner or default period:

```json
{
  "update_config": {
    "owner": "cosmos1newowner...",
    "default_period": 500
  }
}
```

## Query

Read the wallet config:

```json
{ "config": {} }
```

Read one spender's allowance:

```json
{
  "allowance": {
    "spender": "cosmos1spender..."
  }
}
```

Read how much a spender can use in the current period:

```json
{
  "spendable": {
    "spender": "cosmos1spender..."
  }
}
```
