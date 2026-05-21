# Inventory Tracker Contract

This example shows a simple per-wallet inventory tracker.

Each wallet can add inventory items, increase or decrease quantities, update item metadata, delete records, and page through its own inventory. The contract owner controls the maximum number of records each wallet can keep.

## Features

- Independent inventory for each address.
- Sequential item IDs per user.
- SKU, item name, optional note, and quantity fields.
- Increase, decrease, and set quantity flows.
- Safe underflow checks when decreasing quantity.
- Owner-managed maximum item limit per user.
- Paginated item listing and item count queries.

## Instantiate

```json
{
  "max_items_per_user": 100
}
```

## Execute

Add an item:

```json
{
  "add_item": {
    "sku": "HW-WALLET-01",
    "name": "Hardware Wallet",
    "quantity": "2",
    "note": "Cold storage devices"
  }
}
```

Increase quantity:

```json
{
  "increase_quantity": {
    "item_id": 1,
    "amount": "3"
  }
}
```

Decrease quantity:

```json
{
  "decrease_quantity": {
    "item_id": 1,
    "amount": "1"
  }
}
```

Update item metadata:

```json
{
  "update_item": {
    "item_id": 1,
    "sku": "HW-WALLET-02",
    "name": "Hardware Wallet Pro",
    "note": null
  }
}
```

Set quantity directly:

```json
{
  "set_quantity": {
    "item_id": 1,
    "quantity": "10"
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

List a user's inventory:

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
