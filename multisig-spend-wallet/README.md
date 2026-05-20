# Multisig Spend Wallet

This example shows a native-token multisig spending wallet. The contract stores a set of owners and an approval threshold. Owners can propose a native-token spend, other owners can approve it, and the contract sends the funds once the threshold is reached.

## Messages

### Instantiate

```json
{
  "owners": ["cosmos1...", "cosmos1...", "cosmos1..."],
  "threshold": 2
}
```

### Execute

```json
{
  "propose_spend": {
    "recipient": "cosmos1...",
    "amount": { "denom": "uatom", "amount": "100" }
  }
}
```

The proposer is counted as the first approval.

```json
{
  "approve": {
    "proposal_id": 1
  }
}
```

### Query

```json
{ "config": {} }
```

```json
{
  "proposal": {
    "proposal_id": 1
  }
}
```

## What This Demonstrates

- Owner-set validation and threshold checks
- Proposal storage with incrementing IDs
- Preventing duplicate approvals
- Executing native bank sends once a threshold is met
- Querying both wallet config and proposal status
