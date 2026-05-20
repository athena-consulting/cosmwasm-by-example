# Limited Spend Wallet

This example shows a native-token wallet with a delegated spender and a block-height spending limit. The owner can deposit one configured native denomination into the contract, then the delegated spender can send funds to recipients up to the limit for the current window.

When the next block-height window starts, the spent amount resets and the spender can use the allowance again.

## Messages

### Instantiate

```json
{
  "spender": "cosmos1...",
  "denom": "uatom",
  "limit": "100",
  "window_size": 1000
}
```

### Execute

```json
{
  "deposit": {}
}
```

The owner sends native funds with the deposit message.

```json
{
  "spend": {
    "recipient": "cosmos1...",
    "amount": "25"
  }
}
```

Only the configured spender can spend.

```json
{
  "update_spender": {
    "spender": "cosmos1..."
  }
}
```

Only the owner can update the delegated spender.

### Query

```json
{ "config": {} }
```

```json
{ "window": {} }
```

## What This Demonstrates

- Owner/spender authorization
- Native-denomination deposit validation
- Block-height based allowance windows
- Updating stored configuration
- Sending native tokens from a contract
