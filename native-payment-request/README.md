# Native Payment Request

This example shows a simple invoice-style contract for native-token payments.
A merchant creates payment requests, a payer sends the exact native token
amount, and the contract forwards the payment to the configured recipient while
recording the invoice status.

It demonstrates:

- storing sequential or custom invoice IDs
- validating exact native-token payments
- optional payer restrictions and due dates
- canceling unpaid requests
- querying invoice and merchant configuration state

## Flow

1. Instantiate with a payment denom and optional merchant/default recipient.
2. The merchant creates an invoice with an amount, description, optional payer,
   optional recipient, and optional due date.
3. The payer calls `pay_invoice` with the exact coin required by the invoice.
4. The contract forwards the funds to the recipient and marks the invoice paid.
5. Anyone can query invoice state, while only the merchant can create, cancel,
   or update config.

## Messages

### Instantiate

```rust
pub struct InstantiateMsg {
    pub denom: String,
    pub merchant: Option<String>,
    pub default_recipient: Option<String>,
}
```

### Execute

```rust
pub enum ExecuteMsg {
    CreateInvoice {
        id: Option<u64>,
        payer: Option<String>,
        recipient: Option<String>,
        amount: Uint128,
        description: String,
        due_at: Option<u64>,
    },
    PayInvoice {
        id: u64,
    },
    CancelInvoice {
        id: u64,
    },
    UpdateConfig {
        merchant: Option<String>,
        default_recipient: Option<String>,
    },
}
```

### Query

```rust
pub enum QueryMsg {
    Config {},
    Invoice { id: u64 },
    NextInvoiceId {},
}
```
