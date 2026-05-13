# Native RSVP Deposit

This example shows an RSVP deposit contract for events. Attendees reserve a
spot by locking a fixed native-token deposit. The organizer checks in attendees
who arrive, checked-in attendees can claim a refund, and no-show deposits can be
forfeited after the event time.

The contract demonstrates:

- storing owner, event time, denom, deposit amount, and forfeiture recipient
- accepting one RSVP deposit per attendee
- blocking reservations after the event time
- owner-only attendee check-in
- checked-in attendee refunds
- owner-only no-show forfeiture after the event time

## Instantiate

```json
{
  "owner": "cosmos1owner...",
  "forfeiture_recipient": "cosmos1treasury...",
  "denom": "uatom",
  "deposit_amount": "50",
  "event_time": 1735689600
}
```

`owner` is optional. If it is omitted, the sender becomes the owner.
`deposit_amount` must be greater than zero.

## Reserve

```json
{
  "reserve": {}
}
```

The execute message must include exactly one coin with `deposit_amount` of the
configured denom. Each attendee can reserve once before `event_time`.

## Check In

```json
{
  "check_in": {
    "attendee": "cosmos1attendee..."
  }
}
```

Only the owner can mark an attendee as checked in.

## Refund

```json
{
  "refund": {}
}
```

A checked-in attendee can reclaim their deposit. The contract marks the deposit
as settled before sending the refund.

## Forfeit

```json
{
  "forfeit": {
    "attendee": "cosmos1attendee..."
  }
}
```

Only the owner can forfeit a no-show deposit, and only after `event_time`.
Forfeited deposits are sent to `forfeiture_recipient`.

## Query

```json
{ "config": {} }
```

```json
{ "attendee": { "attendee": "cosmos1attendee..." } }
```

Returns contract configuration or an attendee's RSVP state.
