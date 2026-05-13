# Native Savings Goal

This example shows a simple native-token savings goal contract. The contract
stores one denom, one target amount, and one beneficiary. Anyone can fund the
goal with the configured native token. Once the saved balance reaches the target,
the beneficiary can release the entire saved balance to their own address.

## Use Case

This pattern is useful for small community goals, grant milestones, group
payments, or personal savings targets where the release condition is simply
"has the contract collected at least this many native tokens?"

## Messages

- `Fund {}`: accepts exactly one native coin in the configured denom and adds it
  to the saved balance.
- `Release {}`: allows the beneficiary to withdraw all saved funds after the
  target is reached.
- `UpdateBeneficiary { beneficiary }`: allows the owner to rotate the
  beneficiary before the goal is released.

## Safety Checks

- The target amount must be greater than zero.
- Funding must include exactly one native coin.
- Funding must use the configured denom.
- Only the owner can update the beneficiary.
- Only the beneficiary can release funds.
- Funds cannot be released before the target is reached.
- A released goal cannot be funded or released again.

## Query

`Config {}` returns the owner, beneficiary, denom, target amount, saved balance,
and release status.

Run the tests with:

```bash
cargo test --manifest-path native-savings-goal/Cargo.toml
```
