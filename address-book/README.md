# Address Book

This example shows how a CosmWasm contract can keep a small address book in
contract storage.

The contract owner can:

- Add a named contact with a validated blockchain address.
- Remove a contact by name.
- Transfer ownership to another address.

Anyone can query:

- The current owner.
- One contact by name.
- A paginated list of contacts.

The example is intentionally simple. It focuses on common building blocks that
new CosmWasm developers reuse often: `Item` configuration, `Map` storage,
address validation, owner-only execute messages, and paginated range queries.

## Messages

Instantiate with an optional owner. If no owner is provided, the sender becomes
the owner.

```json
{}
```

Add a contact:

```json
{
  "add_contact": {
    "name": "validator-one",
    "address": "cosmos1...",
    "note": "Primary validator address"
  }
}
```

Remove a contact:

```json
{
  "remove_contact": {
    "name": "validator-one"
  }
}
```

Query one contact:

```json
{
  "contact": {
    "name": "validator-one"
  }
}
```

List contacts with optional pagination:

```json
{
  "contacts": {
    "start_after": "validator-one",
    "limit": 10
  }
}
```
