# Native Preorder List

This example shows a native-token preorder list. The owner configures a denom,
unit price, item name, and optional maximum number of units. Buyers place
numbered preorders by sending the configured native token. The owner can mark
orders fulfilled, refund unfulfilled orders, or withdraw collected funds.

## Use Case

This pattern fits small merchandise drops, paid waitlists, event add-ons, or
community sales where the chain should record who prepaid before fulfillment
happens off chain.

## Messages

- `PlaceOrder { quantity, recipient, note }`: accepts native funds and creates a
  numbered preorder. If `recipient` is omitted, the buyer is also the recipient.
- `MarkFulfilled { order_id }`: owner-only message that marks an order fulfilled.
- `RefundOrder { order_id }`: owner-only message that refunds an unfulfilled
  order to the original buyer.
- `Withdraw { recipient, amount }`: owner-only message that withdraws collected
  preorder funds.

## Safety Checks

- The denom and item name cannot be empty.
- Unit price and order quantity must be greater than zero.
- Payments must include exactly one native coin in the configured denom.
- Payments must be at least `unit_price * quantity`.
- Optional max units cannot be exceeded.
- Fulfilled or refunded orders cannot be changed again.
- Withdrawals and refunds cannot exceed the tracked balance.

## Queries

- `Config {}` returns owner, denom, item, unit price, max units, sold units,
  tracked balance, and next order id.
- `Order { order_id }` returns order details.

Run the tests with:

```bash
cargo test --manifest-path native-preorder-list/Cargo.toml
```
