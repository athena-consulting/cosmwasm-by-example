# Native Split Bill

This example shows a simple native-token split-bill contract. A creator opens a
bill with a recipient, a list of participants, and an equal share amount. Each
participant pays exactly one share in the configured native denom. Once every
share is paid, the bill can be settled to the recipient.

## Use Case

This pattern is useful for group purchases, shared event tabs, club expenses,
or any small workflow where several accounts need to pay equal native-token
shares before a merchant or organizer receives the full amount.

## Messages

- `CreateBill { recipient, participants, share_amount, memo }`: creates an open
  bill with one required payment per participant.
- `PayShare { bill_id }`: lets a participant pay their exact share once.
- `Settle { bill_id }`: sends the collected total to the bill recipient after
  every participant has paid.
- `Cancel { bill_id }`: creator-only message that cancels an open bill and
  refunds already-paid participants.

## Safety Checks

- The denom cannot be empty.
- A bill must have at least one unique participant.
- The share amount must be greater than zero.
- Payments must include exactly one native coin in the configured denom.
- A participant can only pay their own share once.
- A bill cannot settle until every participant has paid.
- Only the creator can cancel an open bill.
- Canceled bills refund paid participants from the tracked native balance.

## Queries

- `Config {}` returns owner, denom, next bill id, and tracked balance.
- `Bill { bill_id }` returns bill participants, share, total, paid count, and
  status.
- `ShareStatus { bill_id, participant }` returns whether the participant has
  paid the bill.

Run the tests with:

```bash
cargo test --manifest-path native-split-bill/Cargo.toml
```
