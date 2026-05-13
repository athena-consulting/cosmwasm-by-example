# Native Billboard Rental

This example shows a simple native-token billboard rental contract. The owner
creates labeled display slots, users rent an available slot by paying the
configured native price, and the current renter can update the displayed
message until the rental expires.

## Use Case

This pattern is useful for paid sponsor slots, community notice boards, on-chain
classified ads, app banners, or any workflow where a fixed display resource is
leased for a fixed time window.

## Messages

- `CreateSlot { label }`: owner-only message that creates a rentable display
  slot.
- `RentSlot { slot_id, message }`: pays the configured native price and rents an
  available slot for the configured duration.
- `UpdateMessage { slot_id, message }`: lets the active renter change the slot
  message before expiry.
- `CloseSlot { slot_id }`: owner-only message that disables a slot.
- `Withdraw { recipient, amount }`: owner-only message that withdraws collected
  rental fees.

## Safety Checks

- Denom, price, duration, labels, and messages must be valid.
- Payments must include exactly one native coin in the configured denom.
- A slot cannot be rented again until its paid window expires.
- Only the active renter can update the message before expiry.
- Closed slots cannot be rented.
- Owner withdrawals cannot exceed the tracked native balance.

## Queries

- `Config {}` returns owner, denom, price, duration, next slot id, and balance.
- `Slot { slot_id }` returns label, active status, renter, message, paid-until
  time, and whether the slot is currently available.

Run the tests with:

```bash
cargo test --manifest-path native-billboard-rental/Cargo.toml
```
