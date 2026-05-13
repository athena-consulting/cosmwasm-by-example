# Native Referral Router

This example shows a native-token payment router with an optional dynamic
referrer. Buyers pay the contract, and each payment is routed to the merchant
with an optional referral bonus sent to the supplied referrer.

The contract demonstrates:

- storing owner, merchant, denom, and referral basis points
- validating native token payments
- rejecting self-referrals
- splitting a payment between merchant and referrer
- routing the full payment to the merchant when no referrer is supplied
- owner-only merchant and referral-rate updates

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "merchant": "cosmos1merchant...",
  "denom": "uatom",
  "referral_bps": 1000
}
```

`owner` is optional. If it is omitted, the sender becomes the owner.
`referral_bps` must be between 1 and 10,000.

## Pay

```json
{
  "pay": {
    "referrer": "cosmos1referrer..."
  }
}
```

The execute message must include exactly one positive coin with the configured
denom. If `referrer` is supplied, the contract sends
`amount * referral_bps / 10000` to the referrer and the remainder to the
merchant. If `referrer` is omitted, the full amount is sent to the merchant.

## Update Config

```json
{
  "update_config": {
    "merchant": "cosmos1newmerchant...",
    "referral_bps": 1500
  }
}
```

Only the owner can update the merchant or referral rate. Omitted fields keep
their current values.

## Query

```json
{ "config": {} }
```

Returns owner, merchant, denom, and referral basis points.
