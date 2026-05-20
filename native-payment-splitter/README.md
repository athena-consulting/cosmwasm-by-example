# Native Payment Splitter

This example shows a simple native-token payment splitter. The contract stores a fixed list of recipients and share weights at instantiation. Any caller can send one native coin denomination to `Distribute`, and the contract creates bank send messages that divide the funds by the configured shares.

If integer division leaves dust, the first recipient receives the remainder so all funds are accounted for.

## Messages

### Instantiate

```json
{
  "recipients": [
    { "address": "cosmos1...", "share": "1" },
    { "address": "cosmos1...", "share": "3" }
  ]
}
```

### Execute

```json
{
  "distribute": {}
}
```

The execute call must include exactly one native coin in `funds`.

### Query

```json
{
  "config": {}
}
```

Returns the configured recipients, their shares, and the total share weight.

## What This Demonstrates

- Validating addresses during instantiation
- Storing a vector of payout rules
- Rejecting invalid share configuration
- Building multiple native bank send messages
- Handling integer-division remainder safely
