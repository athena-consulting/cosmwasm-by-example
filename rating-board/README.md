# Rating Board

This example shows how to build a simple rating board in CosmWasm.

Any user can create a named item. Any address can rate that item once, then update its own rating later. The contract stores aggregate totals on each item, so average-rating queries do not need to scan every individual rating.

## What this example teaches

- Creating records with auto-incremented ids
- Using composite keys like `(item_id, rater)`
- Updating aggregate counters when a rating changes
- Querying average values from stored totals
- Paginating item lists
- Owner-only configuration updates

## Messages

### Instantiate

```json
{
  "max_rating": 5
}
```

If omitted, `max_rating` defaults to `5`.

### Create Item

```json
{
  "create_item": {
    "name": "CosmWasm by Example",
    "description": "Learning contracts by reading focused examples"
  }
}
```

### Rate Item

```json
{
  "rate_item": {
    "item_id": 1,
    "rating": 5
  }
}
```

The same rater can call `rate_item` again to update their rating. The stored total is adjusted by the rating delta.

### Update Config

```json
{
  "update_config": {
    "max_rating": 10,
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

### Item

```json
{
  "item": {
    "item_id": 1
  }
}
```

### Rating

```json
{
  "rating": {
    "item_id": 1,
    "rater": "cosmos1..."
  }
}
```

### Items

```json
{
  "items": {
    "start_after": 1,
    "limit": 10
  }
}
```
