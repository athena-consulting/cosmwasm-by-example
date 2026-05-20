# Guestbook

This example shows how to build a small owner-managed guestbook in CosmWasm.

The contract lets any address sign the guestbook with a display name and message. Each address can keep one current entry, edit it by signing again, and remove its own entry. The contract owner can update the guestbook settings or remove any entry if needed.

## What this example teaches

- Storing contract configuration with `Item`
- Storing one record per account with `Map`
- Validating message length before saving state
- Owner-only configuration updates
- Allowing either the signer or owner to delete an entry
- Paginated list queries with `start_after` and `limit`

## Messages

### Instantiate

```json
{
  "title": "Launch party",
  "max_message_len": 140
}
```

`max_message_len` is optional. If omitted, the contract uses `280`.

### Sign

```json
{
  "sign": {
    "display_name": "Ada",
    "message": "Excited to see this project launch."
  }
}
```

### Remove Entry

```json
{
  "remove_entry": {
    "signer": "cosmos1..."
  }
}
```

The signer can remove their own entry. The owner can remove any entry.

### Update Config

```json
{
  "update_config": {
    "title": "New title",
    "max_message_len": 200,
    "new_owner": "cosmos1..."
  }
}
```

Only the owner can update configuration.

## Queries

### Config

```json
{ "config": {} }
```

### Entry

```json
{
  "entry": {
    "signer": "cosmos1..."
  }
}
```

### Entries

```json
{
  "entries": {
    "start_after": "cosmos1...",
    "limit": 10
  }
}
```

The entries query returns signers in address order and caps the page size to keep queries predictable.
