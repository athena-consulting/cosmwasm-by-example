# Native Vesting Wallet

This example shows a native-token vesting wallet. The contract stores one
recipient, one native denom, and a linear release schedule. Funds can be sent
when the contract is instantiated and more funds can be deposited later.

The contract covers:

- validating vesting schedule timestamps
- reading native funds from `MessageInfo`
- calculating claimable vested funds from block time
- sending released native tokens with `BankMsg::Send`
- querying vesting progress and remaining locked funds

## Messages

Instantiate with the recipient, denom, start time, optional cliff, and end time:

```json
{
  "recipient": "cosmos1...",
  "denom": "uatom",
  "start_time": 1700000000,
  "cliff_time": 1700604800,
  "end_time": 1702592000
}
```

Claim any currently vested tokens:

```json
{ "claim": {} }
```

Deposit more vesting funds for the same denom:

```json
{ "deposit": {} }
```

Query the saved config:

```json
{ "config": {} }
```

Query the current vesting status:

```json
{ "status": {} }
```

## Test

```bash
cargo test
```
