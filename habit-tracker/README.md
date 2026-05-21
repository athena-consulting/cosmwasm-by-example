# Habit Tracker

This example shows how to build a simple on-chain habit tracker in CosmWasm.

Each account can check in once a configured number of blocks has passed. The contract stores the user's current streak, best streak, total check-ins, last check-in height, and an optional short note. If a user waits too long between check-ins, the current streak resets while the best streak is preserved.

## What this example teaches

- Storing one record per user address
- Reading `env.block.height` inside execute logic
- Enforcing a block-height cooldown
- Resetting and extending streak counters
- Owner-only configuration updates
- Paginating records by address

## Messages

### Instantiate

```json
{
  "min_gap": 10,
  "max_gap": 100,
  "max_note_len": 140
}
```

If omitted, `min_gap` defaults to `1`, `max_gap` defaults to `100`, and `max_note_len` defaults to `160`.

### Check In

```json
{
  "check_in": {
    "note": "Completed the daily CosmWasm lesson"
  }
}
```
The first check-in starts the user's streak. Later check-ins must wait until at least `min_gap` blocks have passed. If the user checks in after more than `max_gap` blocks, the current streak starts again from `1`.

### Reset

```json
{
  "reset": {}
}
```

Removes the sender's habit record.

### Update Config

```json
{
  "update_config": {
    "min_gap": 20,
    "max_gap": 200,
    "max_note_len": 120,
    "new_owner": "cosmos1..."
  }
}
```

Only the contract owner can update configuration.

## Queries

### Config

```json
{ "config": {} }
```

### Record

```json
{
  "record": {
    "user": "cosmos1..."
  }
}
```

### Records

```json
{
  "records": {
    "start_after": "cosmos1...",
    "limit": 10
  }
}
```
