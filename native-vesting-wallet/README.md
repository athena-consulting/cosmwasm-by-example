# Native Vesting Wallet

This example shows a native-token vesting wallet. The creator funds the contract during instantiation, chooses a beneficiary, sets a cliff height, and defines a linear vesting window. The beneficiary can claim only the vested amount that has not already been claimed.

## Messages

### Instantiate

```json
{
  "beneficiary": "cosmos1...",
  "start_height": 100,
  "cliff_height": 150,
  "end_height": 300
}
```

The instantiate transaction must include exactly one native coin in `funds`.

### Execute

```json
{ "claim": {} }
```

Only the beneficiary can claim.

### Query

```json
{ "vesting": {} }
```

Returns the schedule, total funded amount, amount already claimed, vested amount at the current block height, and amount currently claimable.

## What This Demonstrates

- Funding a contract during instantiation
- Validating a vesting schedule
- Cliff and linear vesting calculations
- Tracking claimed native funds
- Returning height-dependent query data
