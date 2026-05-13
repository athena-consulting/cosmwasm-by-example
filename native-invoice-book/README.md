# Native Invoice Book

This example shows a simple native-token invoice book. The owner creates numbered
invoices with an amount, optional payer, and optional memo. A payer settles an
invoice by sending the configured native denom. Paid invoices are recorded on
chain, and the owner can withdraw collected invoice funds.

## Use Case

This pattern is useful for small service providers, grant operators, or DAOs that
want a lightweight record of native-token invoices without adding CW20 logic.

## Messages

- `CreateInvoice { payer, amount, memo }`: owner-only message that creates a new
  unpaid invoice and returns its id in the response attributes.
- `PayInvoice { invoice_id }`: accepts native funds and marks the invoice paid.
  If the invoice has a configured payer, only that address can pay it.
- `CancelInvoice { invoice_id }`: owner-only message that removes an unpaid
  invoice.
- `Withdraw { recipient, amount }`: owner-only message that sends collected
  invoice funds to a recipient.

## Safety Checks

- The denom cannot be empty.
- Invoice amounts must be greater than zero.
- Invoice payments must include exactly one native coin in the configured denom.
- Invoice payments must be at least the invoice amount.
- Paid invoices cannot be paid or canceled again.
- Only the owner can create, cancel, or withdraw.
- Withdrawals cannot exceed the tracked collected balance.

## Queries

- `Config {}` returns owner, denom, collected balance, and next invoice id.
- `Invoice { invoice_id }` returns invoice details.

Run the tests with:

```bash
cargo test --manifest-path native-invoice-book/Cargo.toml
```
