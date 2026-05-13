# Native Event Ticket

This example shows a native-token event ticket sale. Buyers pay a fixed native
token price for a limited number of tickets, and the organizer can later
withdraw collected revenue.

The contract demonstrates:

- storing ticket price, supply cap, organizer, and revenue accounting
- validating exact native-token payments
- preventing purchases above the supply cap
- tracking ticket quantities per buyer
- owner-only revenue withdrawals
- owner-only organizer updates

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "organizer": "cosmos1organizer...",
  "denom": "uatom",
  "price": "25",
  "max_tickets": 100
}
```

`owner` is optional. If it is omitted, the sender becomes the owner. `price`
and `max_tickets` must both be greater than zero.

## Buy

```json
{
  "buy": {
    "quantity": 2
  }
}
```

The execute message must include exactly `price * quantity` of the configured
denom. The contract increments the buyer's ticket count and the total sold
count.

## Withdraw

```json
{
  "withdraw": {
    "recipient": "cosmos1treasury...",
    "amount": "500"
  }
}
```

Only the owner can withdraw revenue. If `recipient` is omitted, funds go to the
configured organizer. If `amount` is omitted, the full revenue balance is
withdrawn.

## Update Organizer

```json
{
  "update_organizer": {
    "organizer": "cosmos1neworganizer..."
  }
}
```

Only the owner can update the default organizer withdrawal address.

## Query

```json
{ "config": {} }
```

```json
{ "tickets": { "buyer": "cosmos1buyer..." } }
```

Returns sale configuration or the ticket quantity held by a buyer.
