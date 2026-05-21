# Simple Poll

This example shows a small poll contract that lets any account create a poll, lets each address vote once, and closes voting by block height or by an authorized closer.

It is intentionally compact so new CosmWasm developers can see how to combine:

- validated instantiate-time configuration
- generated poll IDs
- a `Map` keyed by poll ID
- one-vote-per-address tracking
- block-height based voting windows
- paginated list queries

## Instantiate

All instantiate fields are optional. The sender becomes the owner and can later update limits.

```json
{
  "max_question_len": 160,
  "max_option_len": 64,
  "max_options": 8
}
```

## Create A Poll

Any account can create a poll. A poll needs at least two unique options. `closes_at` is optional, but when present it must be a future block height.

```json
{
  "create_poll": {
    "question": "Which feature should ship first?",
    "options": ["Dark mode", "CSV export", "Mobile layout"],
    "closes_at": 500000
  }
}
```

## Vote

Each address can vote once per poll. Option indexes start at `0`.

```json
{
  "vote": {
    "poll_id": 1,
    "option_index": 0
  }
}
```

## Close A Poll

The poll creator or contract owner can close a poll manually.

```json
{
  "close_poll": {
    "poll_id": 1
  }
}
```

## Queries

Read a single poll:

```json
{
  "poll": {
    "poll_id": 1
  }
}
```

List polls with pagination:

```json
{
  "polls": {
    "start_after": 1,
    "limit": 10
  }
}
```

Check how an address voted:

```json
{
  "vote": {
    "poll_id": 1,
    "voter": "cosmos1..."
  }
}
```

## Validation

Run the contract tests from this directory:

```sh
cargo test
```
