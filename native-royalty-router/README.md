# Native Royalty Router

This example shows a native-token royalty router. The owner configures a list of
recipients and basis-point shares that sum to 10,000. When a payer routes a
payment, the contract immediately forwards the native tokens according to those
shares.

The contract demonstrates:

- storing owner, denom, and royalty share configuration
- validating basis-point shares and duplicate recipients
- validating exactly one native coin on each routed payment
- splitting a payment with integer math
- assigning rounding remainder to the first configured recipient
- owner-only share updates

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "denom": "uatom",
  "shares": [
    { "recipient": "cosmos1artist...", "bps": 7000 },
    { "recipient": "cosmos1producer...", "bps": 3000 }
  ]
}
```

`owner` is optional. If it is omitted, the sender becomes the owner.

## Route Payment

```json
{
  "route_payment": {}
}
```

The execute message must include exactly one positive coin with the configured
denom. The contract calculates each recipient amount and emits native bank send
messages in the same transaction.

## Update Shares

```json
{
  "update_shares": {
    "shares": [
      { "recipient": "cosmos1artist...", "bps": 8000 },
      { "recipient": "cosmos1publisher...", "bps": 2000 }
    ]
  }
}
```

Only the owner can update shares. New shares must sum to exactly 10,000 basis
points.

## Query

```json
{ "config": {} }
```

Returns the owner, denom, and configured royalty shares.
