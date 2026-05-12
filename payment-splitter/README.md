# Payment Splitter

This example shows a native-token payment splitter contract. It stores a fixed
set of members and relative shares at instantiation time, then splits each
incoming payment by those shares.

The contract covers:

- validating addresses and share weights during instantiation
- storing reusable configuration with `cw-storage-plus`
- sending native tokens with `BankMsg::Send`
- handling integer division remainders deterministically
- exposing query endpoints for config and split previews

## Messages

Instantiate the contract with a denom, members, and an optional remainder
recipient:

```json
{
  "denom": "uatom",
  "members": [
    { "address": "cosmos1...", "share": 2 },
    { "address": "cosmos1...", "share": 1 },
    { "address": "cosmos1...", "share": 1 }
  ],
  "remainder_recipient": "cosmos1..."
}
```

Execute the split by sending funds and calling:

```json
{ "distribute": {} }
```

Query the saved configuration:

```json
{ "config": {} }
```

Preview a split before executing it:

```json
{ "preview": { "amount": "1000" } }
```

## Test

```bash
cargo test
```
