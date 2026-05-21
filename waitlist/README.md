# Waitlist

This example shows a small first-come waitlist contract. It is useful for
product launches, allowlist admissions, event queues, beta access programs, or
any other flow where addresses join a queue and an owner later admits or removes
entries.

The contract demonstrates:

- storing one active waitlist entry per address
- assigning monotonic queue positions
- validating optional notes
- owner-only admission and removal
- accurate waiting/admitted counters
- ownership transfer and configurable limits
- paginated listing of waitlist entries

## Messages

Instantiate the contract with optional limits:

```json
{
  "max_note_length": 120,
  "max_entries": 500
}
```

Join the waitlist:

```json
{
  "join": {
    "note": "Interested in beta access"
  }
}
```

Update your note:

```json
{
  "update_note": {
    "note": "Prefer the next available cohort"
  }
}
```

Leave the waitlist:

```json
{
  "leave": {}
}
```

Admit a waiting member:

```json
{
  "admit": {
    "member": "cosmos1..."
  }
}
```

Remove a member:

```json
{
  "remove": {
    "member": "cosmos1..."
  }
}
```

Update limits or transfer ownership:

```json
{
  "update_config": {
    "owner": "cosmos1newowner...",
    "max_note_length": 160,
    "max_entries": 1000
  }
}
```

## Queries

Get contract configuration:

```json
{
  "config": {}
}
```

Get an address entry:

```json
{
  "entry": {
    "member": "cosmos1..."
  }
}
```

Get aggregate counts:

```json
{
  "stats": {}
}
```

List entries with optional pagination:

```json
{
  "list_entries": {
    "start_after": "cosmos1...",
    "limit": 20
  }
}
```
