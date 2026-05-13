# Native Warranty Registry

This example shows a simple native-token warranty registry. Users pay a native
fee to register a product serial for a fixed coverage period, transfer product
ownership, file warranty claims, and let the contract owner resolve claims.

## Use Case

This pattern is useful for paid warranty cards, hardware registration,
after-sales support records, repair intake, or any workflow where native-token
fees fund a time-limited product coverage registry.

## Messages

- `RegisterProduct { serial, metadata }`: pays the configured native fee and
  registers an available or expired serial.
- `TransferProduct { serial, new_owner }`: current product owner transfers the
  covered product to another account.
- `FileClaim { serial, note }`: current product owner opens a warranty claim
  while coverage is active.
- `ResolveClaim { claim_id, approved }`: contract owner resolves an open claim.
- `Withdraw { recipient, amount }`: contract owner withdraws collected fees.

## Safety Checks

- Denom, fee, coverage period, serial, metadata, and claim note must be valid.
- Payments must include exactly one native coin in the configured denom.
- Active product registrations cannot be overwritten by another user.
- Only the current product owner can transfer a product or file a claim.
- Claims cannot be filed after warranty coverage expires.
- Only the contract owner can resolve claims or withdraw collected fees.

## Queries

- `Config {}` returns owner, denom, fee, coverage period, next claim id, and
  tracked native balance.
- `Product { serial }` returns owner, metadata, expiry, and expired status.
- `Claim { claim_id }` returns serial, claimant, note, and claim status.

Run the tests with:

```bash
cargo test --manifest-path native-warranty-registry/Cargo.toml
```
