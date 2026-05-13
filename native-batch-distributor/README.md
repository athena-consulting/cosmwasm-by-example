# Native Batch Distributor

This example shows a small native-token distributor contract. An admin chooses
one denom during instantiation, funds the contract with that denom, and sends a
batch of fixed payments in one transaction.

The contract demonstrates:

- storing admin and denom configuration
- validating native funds sent with an execute message
- checking admin-only permissions
- validating recipient addresses before sending funds
- rejecting empty batches, zero payments, and duplicate recipients
- querying the contract's native balance before building `BankMsg::Send` messages

## Instantiate

```json
{
  "admin": "cosmos1admin...",
  "denom": "uatom"
}
```

`admin` is optional. If it is omitted, the sender becomes the admin.

## Deposit

```json
{
  "deposit": {}
}
```

The message must include exactly one positive coin with the configured denom.
Deposits only move native funds into the contract account; no per-user ledger is
stored.

## Distribute

```json
{
  "distribute": {
    "payments": [
      {
        "recipient": "cosmos1alice...",
        "amount": "100"
      },
      {
        "recipient": "cosmos1bob...",
        "amount": "250"
      }
    ]
  }
}
```

Only the admin can distribute funds. The contract validates each recipient,
rejects duplicate recipients in the same batch, sums the requested amount, checks
the contract balance, and then emits one bank send message per payment.

## Update Admin

```json
{
  "update_admin": {
    "admin": "cosmos1newadmin..."
  }
}
```

The current admin can transfer control to a new admin address.

## Queries

```json
{ "config": {} }
```

Returns the admin and configured denom.

```json
{ "balance": {} }
```

Returns the contract's native balance for the configured denom.
