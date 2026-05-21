# Wishlist Contract

This example shows a simple personal wishlist contract.

Each wallet can add, update, purchase-mark, and delete its own wish items. The contract keeps a per-user sequence so item IDs are easy to query, and it includes pagination for user wishlists.

## Features

- One independent wishlist per address.
- Sequential item IDs for each user.
- Optional description and URL fields.
- Priority values from 1 to 5.
- Purchased/unpurchased status updates.
- Owner-managed maximum item limit per user.
- Paginated item listing.

## Instantiate

```json
{
  "max_items_per_user": 50
}
```

## Execute

Add an item:

```json
{
  "add_item": {
    "title": "Ledger hardware wallet",
    "description": "Cold-storage device for long-term funds",
    "url": "https://example.com/ledger",
    "priority": 5
  }
}
```

Mark an item as purchased:

```json
{
  "set_purchased": {
    "item_id": 1,
    "purchased": true
  }
}
```

Update an item:

```json
{
  "update_item": {
    "item_id": 1,
    "title": "Ledger hardware wallet",
    "description": "Prefer the black model",
    "url": null,
    "priority": 4
  }
}
```

Delete an item:

```json
{
  "delete_item": {
    "item_id": 1
  }
}
```

## Query

List a user's wishlist:

```json
{
  "user_items": {
    "owner": "cosmos1owner...",
    "start_after": null,
    "limit": 10
  }
}
```

Query one item:

```json
{
  "item": {
    "owner": "cosmos1owner...",
    "item_id": 1
  }
}
```
