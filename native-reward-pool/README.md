# Native Reward Pool

This example shows a native-token reward pool. The owner funds the contract,
assigns claimable rewards to addresses, and recipients pull their own rewards
when ready.

The contract demonstrates:

- storing owner, denom, and total allocated reward state
- validating one configured native denom for deposits
- owner-only reward allocation
- preventing allocations above the current pool balance
- recipient-driven reward claims
- updating an allocation ledger before emitting bank sends

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "denom": "uatom"
}
```

`owner` is optional. If it is omitted, the sender becomes the owner.

## Deposit

```json
{
  "deposit": {}
}
```

The execute message must include exactly one positive coin with the configured
denom. Deposits increase the contract account balance.

## Set Reward

```json
{
  "set_reward": {
    "recipient": "cosmos1recipient...",
    "amount": "100"
  }
}
```

Only the owner can set rewards. The contract replaces any existing unclaimed
reward for that recipient and verifies that total allocated rewards do not exceed
the contract balance.

## Claim

```json
{
  "claim": {}
}
```

The sender claims their assigned reward. A successful claim clears the reward
entry and sends the native tokens to the sender.

## Queries

```json
{ "config": {} }
```

Returns owner, denom, and total allocated amount.

```json
{ "reward": { "recipient": "cosmos1recipient..." } }
```

Returns the current unclaimed reward for one recipient.

```json
{ "pool": {} }
```

Returns the contract balance and total allocated amount.
