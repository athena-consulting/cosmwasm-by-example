# Native Name Registry

This example shows a simple native-token name registry. Users pay a configured
native registration fee to claim a normalized name for a fixed period, update
the associated value while the registration is active, renew the name, or
release it.

## Use Case

This pattern is useful for usernames, short links, profile handles, service
aliases, or any lightweight registry where a native-token fee prevents
unbounded name squatting.

## Messages

- `Register { name, value }`: pays the registration fee and claims an available
  or expired name.
- `Renew { name }`: current owner pays the fee to extend an active or expired
  registration.
- `UpdateValue { name, value }`: current owner updates the registry value while
  the name is active.
- `Release { name }`: current owner deletes the record and makes the name
  available again.
- `Withdraw { recipient, amount }`: contract owner withdraws collected native
  fees.

## Safety Checks

- Denom, fee, registration period, name, and value must be valid.
- Payments must include exactly one native coin in the configured denom.
- Registration and renewal payments must cover the configured fee.
- A name cannot be registered by another account until it expires or is
  released.
- Only the current name owner can renew, update, or release a name.
- Contract-owner withdrawals cannot exceed the tracked native balance.

## Queries

- `Config {}` returns owner, denom, fee, registration period, and tracked
  balance.
- `Name { name }` returns the normalized name, owner, value, expiry, and whether
  the record is expired.

Run the tests with:

```bash
cargo test --manifest-path native-name-registry/Cargo.toml
```
