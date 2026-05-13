# Native Budget Wallet

This example shows a simple native-token budget wallet. An owner funds the
wallet, creates named budget lines with managers and caps, and managers claim
native-token payouts until their budget line is exhausted.

## Use Case

This pattern is useful for DAO working groups, project departments, grant
milestones, hackathon tracks, or small teams that want a single funded wallet
with separately tracked spending caps.

## Messages

- `Fund {}`: accepts native tokens in the configured denom and increases the
  wallet balance.
- `CreateBudget { manager, label, cap }`: owner-only message that creates a
  named budget line with a spending cap.
- `ClaimBudget { budget_id, recipient, amount, memo }`: lets the budget manager
  claim part of the budget to a recipient; if omitted, the manager receives it.
- `CloseBudget { budget_id }`: owner-only message that closes a budget line.
- `WithdrawUnreserved { recipient, amount }`: owner-only message that withdraws
  balance not reserved for active budget lines.

## Safety Checks

- The denom and budget label cannot be empty.
- Funding must include exactly one native coin in the configured denom.
- Budget caps and claimed amounts must be greater than zero.
- Only the owner can create or close budgets and withdraw unreserved funds.
- Only the budget manager can claim from a budget line.
- Claims cannot exceed the line's remaining cap or the wallet's tracked balance.
- Owner withdrawals cannot consume funds reserved for active budgets.

## Queries

- `Config {}` returns owner, denom, next budget id, balance, reserved amount, and
  unreserved amount.
- `Budget { budget_id }` returns manager, label, cap, spent, remaining, and
  active status.

Run the tests with:

```bash
cargo test --manifest-path native-budget-wallet/Cargo.toml
```
