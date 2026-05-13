# Native Payroll Wallet

This example shows a native-token payroll wallet. An owner configures employee
salaries, funds the payroll balance, and employees claim their salary after each
configured period.

The contract demonstrates:

- storing owner, denom, payroll period, and funded balance
- accepting native-token payroll funding
- owner-only employee salary configuration
- owner-only employee removal
- employee self-claim after a claim period elapses
- balance checks before salary payout

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "denom": "uatom",
  "period_seconds": 604800
}
```

`owner` is optional. If it is omitted, the sender becomes the owner.
`period_seconds` must be greater than zero.

## Fund

```json
{
  "fund": {}
}
```

The execute message must include exactly one positive coin with the configured
denom. The amount is added to the payroll balance.

## Set Employee

```json
{
  "set_employee": {
    "employee": "cosmos1employee...",
    "salary": "125",
    "last_claim": 1735689600
  }
}
```

Only the owner can set or update an employee salary. `last_claim` is optional
and defaults to `0`.

## Remove Employee

```json
{
  "remove_employee": {
    "employee": "cosmos1employee..."
  }
}
```

Only the owner can remove an employee record.

## Claim

```json
{
  "claim": {}
}
```

An employee can claim when the current block time is at least
`last_claim + period_seconds`. The contract sends the configured salary and
updates `last_claim` to the current block time.

## Query

```json
{ "config": {} }
```

```json
{ "employee": { "employee": "cosmos1employee..." } }
```

Returns payroll configuration or employee claim state.
