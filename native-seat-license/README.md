# Native Seat License

This example shows a simple native-token seat license contract. The owner sets
a fixed seat supply and native price. Users buy transferable seats for
themselves or another account, while the owner can revoke seats and withdraw
collected fees.

## Use Case

This pattern is useful for small software licenses, limited beta seats, private
community access, vendor seats, or any workflow where access is represented by
a finite number of paid native-token seats.

## Messages

- `BuySeat { account }`: pays the configured native price and assigns a seat to
  `account`; if omitted, the sender receives the seat.
- `TransferSeat { recipient }`: lets a seat holder transfer their seat.
- `RevokeSeat { account }`: owner-only message that removes a seat and frees one
  unit of supply.
- `Withdraw { recipient, amount }`: owner-only message that withdraws collected
  native fees.

## Safety Checks

- Denom, price, and maximum seat supply must be valid.
- Payments must include exactly one native coin in the configured denom.
- Purchases cannot exceed the fixed seat supply.
- An account can only hold one seat.
- Only a current seat holder can transfer a seat.
- Only the owner can revoke seats or withdraw collected fees.

## Queries

- `Config {}` returns owner, denom, seat price, max seats, sold seats, and
  tracked native balance.
- `Seat { account }` returns whether an account currently owns a seat.

Run the tests with:

```bash
cargo test --manifest-path native-seat-license/Cargo.toml
```
