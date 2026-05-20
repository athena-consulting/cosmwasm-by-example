# Native Claim Voucher

This example shows how to hold native tokens in a simple claim voucher.

A creator deposits native funds into a voucher for one recipient. The recipient
can claim the funds before the expiry block height. If the voucher expires
unclaimed, the creator can refund the locked funds.

## Messages

### Instantiate

```json
{}
```

### CreateVoucher

Creates a voucher and locks the native funds sent with the execute message.

```json
{
  "create_voucher": {
    "id": "invoice-1",
    "recipient": "cosmos1recipient...",
    "expires_at_height": 20000
  }
}
```

Rules:

- The message must include at least one native coin.
- Voucher IDs are unique.
- The recipient address is validated before the voucher is stored.

### Claim

Pays the voucher funds to the recipient.

```json
{
  "claim": {
    "id": "invoice-1"
  }
}
```

Rules:

- Only the configured recipient can claim.
- The voucher must still be open.
- The current block height must be less than or equal to `expires_at_height`.

### RefundExpired

Pays an expired, unclaimed voucher back to the creator.

```json
{
  "refund_expired": {
    "id": "invoice-1"
  }
}
```

Rules:

- Only the creator can refund.
- The voucher must still be open.
- The current block height must be greater than `expires_at_height`.

## Query

```json
{
  "voucher": {
    "id": "invoice-1"
  }
}
```

Returns the creator, recipient, locked funds, expiry height, and one of these
statuses: `open`, `claimed`, or `refunded`.

## Run Tests

```bash
cargo test
```

The tests cover recipient claim, unauthorized claim rejection, and creator
refund after expiry.
