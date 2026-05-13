# Native Reimbursement Claim

This example shows a simple native-token reimbursement contract. The owner funds
the contract, approves one-time reimbursement claims for specific claimants, and
each claimant can claim their approved amount once.

## Use Case

This pattern fits small grants, DAO contributor expenses, travel reimbursements,
or team spending workflows where an administrator approves a fixed native-token
amount after reviewing receipts off chain.

## Messages

- `Fund {}`: accepts native funds in the configured denom and increases the
  reimbursement pool balance.
- `ApproveClaim { claimant, amount, description }`: owner-only message that
  creates a numbered reimbursement claim.
- `Claim { claim_id }`: lets the approved claimant withdraw the approved amount
  once.
- `CancelClaim { claim_id }`: owner-only message that removes an unpaid claim.
- `WithdrawExcess { recipient, amount }`: owner-only message for returning
  unused pool funds.

## Safety Checks

- The denom cannot be empty.
- Funding must include exactly one native coin in the configured denom.
- Claim amounts must be greater than zero.
- Only the owner can approve, cancel, or withdraw.
- Only the approved claimant can claim.
- A paid claim cannot be claimed or canceled again.
- Claims and excess withdrawals cannot exceed the tracked pool balance.

## Queries

- `Config {}` returns owner, denom, pool balance, and next claim id.
- `Claim { claim_id }` returns claim details.

Run the tests with:

```bash
cargo test --manifest-path native-reimbursement-claim/Cargo.toml
```
