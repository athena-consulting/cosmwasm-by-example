# Native Gift Card

This example shows a native-token gift card contract. A creator locks a fixed
amount of one denom for a recipient. The recipient can claim the gift, while the
creator can cancel and refund an unclaimed gift.

The contract demonstrates:

- storing denom configuration and auto-incrementing IDs
- validating native funds on gift creation
- saving per-gift creator, recipient, amount, and status
- restricting claim to the gift recipient
- restricting cancellation to the gift creator
- emitting native bank sends for claim and cancellation paths

## Instantiate

```json
{
  "denom": "uatom"
}
```

## Create Gift

```json
{
  "create_gift": {
    "recipient": "cosmos1friend..."
  }
}
```

The execute message must include exactly one positive coin with the configured
denom. The contract assigns the next `gift_id` and records the recipient.

## Claim

```json
{
  "claim": {
    "gift_id": 1
  }
}
```

Only the gift recipient can claim. A successful claim sends the gift amount to
the recipient and marks the gift as claimed.

## Cancel

```json
{
  "cancel": {
    "gift_id": 1
  }
}
```

Only the gift creator can cancel an unclaimed gift. A successful cancellation
refunds the gift amount to the creator and marks the gift as canceled.

## Queries

```json
{ "config": {} }
```

Returns the configured denom and next gift ID.

```json
{ "gift": { "gift_id": 1 } }
```

Returns the stored gift record and its current status.
