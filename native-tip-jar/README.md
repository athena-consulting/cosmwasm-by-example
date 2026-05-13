# Native Tip Jar

This example shows a small native-token tip jar. The contract stores one owner
and one accepted denom. Anyone can send a tip in that denom, and the owner can
withdraw part or all of the contract balance to a chosen recipient.

The contract demonstrates:

- storing owner and denom configuration
- validating that execute funds contain exactly one positive native coin
- rejecting tips with the wrong denom or multiple denoms
- querying the contract's native balance before withdrawal
- limiting withdrawals and owner transfers to the current owner

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "denom": "uatom"
}
```

`owner` is optional. If it is omitted, the sender becomes the owner.

## Tip

```json
{
  "tip": {}
}
```

The message must include exactly one positive coin with the configured denom.
The contract does not track per-tipper balances; tips simply accumulate in the
contract account.

## Withdraw

```json
{
  "withdraw": {
    "amount": "500",
    "recipient": "cosmos1creator..."
  }
}
```

Only the owner can withdraw. `amount` and `recipient` are optional. If `amount`
is omitted, the contract withdraws the full configured-denom balance. If
`recipient` is omitted, funds are sent to the owner.

## Update Owner

```json
{
  "update_owner": {
    "owner": "cosmos1newowner..."
  }
}
```

The current owner can transfer control to a new owner address.

## Queries

```json
{ "config": {} }
```

Returns the owner and accepted denom.

```json
{ "balance": {} }
```

Returns the contract's native balance for the accepted denom.
