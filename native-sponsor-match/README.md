# Native Sponsor Match

This example shows a native-token donation matching contract. Sponsors fund a
matching pool. When donors send a configured denom, the contract forwards the
donation plus a calculated match to the beneficiary.

The contract demonstrates:

- storing owner, beneficiary, denom, match rate, and pool accounting
- accepting native funds into a match pool
- validating donor payments
- calculating a basis-point match amount
- rejecting donations when the match pool is underfunded
- owner-only beneficiary and match-rate updates

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "beneficiary": "cosmos1beneficiary...",
  "denom": "uatom",
  "match_bps": 5000
}
```

`owner` is optional. If it is omitted, the sender becomes the owner. `match_bps`
must be between 1 and 10,000.

## Fund Match Pool

```json
{
  "fund_match_pool": {}
}
```

The execute message must include exactly one positive coin with the configured
denom. This increases the tracked match pool.

## Donate

```json
{
  "donate": {}
}
```

The execute message must include exactly one positive coin with the configured
denom. The contract calculates `donation * match_bps / 10000`, subtracts that
match from the pool, and sends donation plus match to the beneficiary.

## Update Config

```json
{
  "update_config": {
    "beneficiary": "cosmos1newbeneficiary...",
    "match_bps": 2500
  }
}
```

Only the owner can update the beneficiary or match rate. Omitted fields keep
their current values.

## Query

```json
{ "config": {} }
```

Returns owner, beneficiary, denom, match rate, and remaining match pool.
