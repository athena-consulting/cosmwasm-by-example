# Native Membership Dues

This example shows a small membership-dues contract that accepts native-token
payments and tracks membership expiration times. A member pays the configured
fee to extend their active membership by a fixed period. The owner can withdraw
collected dues to a chosen recipient.

## Use Case

This pattern can support simple clubs, private communities, gated DAO workspaces,
or recurring access passes where membership is represented by on-chain state and
fees are paid in a native denom.

## Messages

- `PayDues { member }`: accepts the configured membership fee. If `member` is
  omitted, the sender pays for their own membership. If a membership is already
  active, the new period is added after the existing expiration.
- `Withdraw { recipient, amount }`: lets the owner withdraw collected dues from
  the contract balance.
- `UpdateTerms { fee, period_seconds }`: lets the owner update future dues terms.

## Safety Checks

- The denom cannot be empty.
- The fee and membership period must be greater than zero.
- A dues payment must include exactly one native coin in the configured denom.
- A dues payment must be at least the configured fee.
- Only the owner can withdraw funds or update terms.
- Withdrawals cannot exceed the tracked contract balance.

## Queries

- `Config {}` returns owner, denom, fee, period, and tracked balance.
- `Member { member }` returns whether the member is currently active and when
  their membership expires.

Run the tests with:

```bash
cargo test --manifest-path native-membership-dues/Cargo.toml
```
