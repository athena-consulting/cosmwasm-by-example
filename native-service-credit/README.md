# Native Service Credit

This example shows a simple native-token service-credit ledger. Users buy
internal credits by sending the configured native token. Credits are tracked in
contract storage, can be transferred between users, and can be debited by the
owner when an off-chain service is delivered.

## Use Case

This pattern is useful for pay-as-you-go API credits, support credits, compute
credits, or small service shops that want native-token payments but account
usage with an internal credit balance.

## Messages

- `BuyCredits { account, credits }`: accepts native funds and mints internal
  credits to `account`; if omitted, credits go to the sender.
- `TransferCredits { recipient, credits }`: lets a user transfer unused credits.
- `DebitCredits { account, credits, reason }`: owner-only message that reduces a
  user's balance when service is consumed.
- `Withdraw { recipient, amount }`: owner-only message that withdraws collected
  native funds.

## Safety Checks

- The denom cannot be empty.
- Unit price and credit amounts must be greater than zero.
- Payments must include exactly one native coin in the configured denom.
- Payments must be at least `unit_price * credits`.
- Users cannot transfer more credits than they own.
- Only the owner can debit credits or withdraw collected funds.
- Withdrawals cannot exceed the tracked native balance.

## Queries

- `Config {}` returns owner, denom, unit price, and tracked native balance.
- `Credit { account }` returns the account's current internal credit balance.

Run the tests with:

```bash
cargo test --manifest-path native-service-credit/Cargo.toml
```
