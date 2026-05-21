# Dead Man Switch Vault

The Dead Man Switch Vault contract demonstrates a simple inheritance-style native token vault. An owner keeps the vault alive by calling `Heartbeat`. If the owner misses the configured block-height window, the configured beneficiary can claim the contract's balance in the configured native denom.

This example demonstrates:

- owner and beneficiary authorization checks
- block-height based liveness windows
- receiving native funds with an execute message
- sending native funds from a contract with `BankMsg::Send`
- querying contract status without changing state

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "beneficiary": "cosmos1beneficiary...",
  "denom": "uatom",
  "heartbeat_window": 10000
}
```

`owner` is optional. If omitted, the instantiator becomes the owner.

## Execute

### Deposit

```json
{
  "deposit": {}
}
```

Attach native tokens in the configured denom. Anyone can deposit.

### Heartbeat

```json
{
  "heartbeat": {}
}
```

Only the owner can call this. It updates the last heartbeat height to the current block height.

### Update Config

```json
{
  "update_config": {
    "owner": "cosmos1newowner...",
    "beneficiary": "cosmos1newbeneficiary...",
    "heartbeat_window": 20000
  }
}
```

Only the owner can call this. Every field is optional.

### Claim

```json
{
  "claim": {
    "recipient": "cosmos1recipient...",
    "amount": "250000"
  }
}
```

Only the beneficiary can claim, and only after the heartbeat window has expired. `recipient` and `amount` are optional. If no recipient is provided, funds go to the beneficiary. If no amount is provided, the whole contract balance for the configured denom is claimed.

## Query

### Config

```json
{
  "config": {}
}
```

### Status

```json
{
  "status": {}
}
```

Returns the owner, beneficiary, configured denom, heartbeat window, last heartbeat height, expiration height, and whether the vault is currently expired.
