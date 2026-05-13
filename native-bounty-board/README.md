# Native Bounty Board

This example shows a native-token bounty board. A sponsor creates a funded
bounty, then either awards the locked funds to a recipient or cancels the bounty
to refund the sponsor.

The contract demonstrates:

- storing a configured native denom
- creating numbered bounty records
- validating exactly one positive native payment
- locking the sponsor's funds until settlement
- sponsor-only award and cancel flows
- querying individual bounties and the next id

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "denom": "uatom"
}
```

`owner` is optional. If it is omitted, the sender becomes the owner. `denom`
must be a non-empty native token denom.

## Create Bounty

```json
{
  "create_bounty": {
    "title": "Fix wallet bug"
  }
}
```

The execute message must include exactly one positive coin with the configured
denom. The contract stores the sender as sponsor and locks the funds.

## Award

```json
{
  "award": {
    "id": 1,
    "recipient": "cosmos1solver..."
  }
}
```

Only the sponsor can award an unsettled bounty. Awarding sends the locked funds
to `recipient` and marks the bounty as settled.

## Cancel

```json
{
  "cancel": {
    "id": 1
  }
}
```

Only the sponsor can cancel an unsettled bounty. Canceling refunds the locked
funds to the sponsor and marks the bounty as settled.

## Query

```json
{ "config": {} }
```

```json
{ "bounty": { "id": 1 } }
```

```json
{ "next_id": {} }
```

Returns the config, an individual bounty, or the next bounty id.
