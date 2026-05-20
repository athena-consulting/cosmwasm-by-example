# Binary Poll

This example shows a simple yes/no poll. The creator sets a question and closing block height at instantiation. Any address can vote once before the poll closes, and queries return the live tally plus an individual voter's ballot.

## Messages

### Instantiate

```json
{
  "question": "Should the proposal pass?",
  "closes_at_height": 123456
}
```

### Execute

```json
{
  "vote": {
    "approve": true
  }
}
```

Each address can vote only once.

### Query

```json
{ "poll": {} }
```

```json
{
  "vote": {
    "voter": "cosmos1..."
  }
}
```

## What This Demonstrates

- Storing poll metadata and tallies
- Enforcing a block-height voting deadline
- Preventing duplicate votes with a map
- Returning optional per-voter query data
- Deriving a simple result label from state
