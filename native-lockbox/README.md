# Native Lockbox

This example shows a multi-user native-token lockbox. Any user can create a
numbered lock with a future unlock time, optionally extend it, and claim their
funds once the unlock time has passed.

The contract demonstrates:

- storing a configured native denom
- creating numbered lock records
- validating positive native-token deposits
- enforcing future unlock times
- owner-only lock extension
- owner-only claims after unlock time

## Instantiate

```json
{
  "denom": "uatom"
}
```

`denom` must be a non-empty native token denom.

## Lock

```json
{
  "lock": {
    "unlock_time": 1735689600
  }
}
```

The execute message must include exactly one positive coin with the configured
denom. `unlock_time` must be greater than the current block time.

## Extend

```json
{
  "extend": {
    "id": 1,
    "unlock_time": 1735776000
  }
}
```

Only the lock owner can extend an unclaimed lock, and the new unlock time must
be later than the current unlock time.

## Claim

```json
{
  "claim": {
    "id": 1
  }
}
```

Only the lock owner can claim. The current block time must be at or after the
lock's unlock time.

## Query

```json
{ "config": {} }
```

```json
{ "lock": { "id": 1 } }
```

```json
{ "next_id": {} }
```

Returns the config, an individual lock, or the next lock id.
