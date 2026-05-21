# Profile Card

This contract stores a small public profile for each wallet address. It is a
simple example of owner-controlled state: every sender can create, update, query,
list, or clear only their own profile card.

The example demonstrates:

- validating execute message fields before saving state
- using `Map` to store one record per address
- removing state with `Map::remove`
- paginating query results with `start_after` and `limit`

## Instantiate

The contract does not need any setup data.

```rust
pub struct InstantiateMsg {}
```

## Execute Messages

`SetProfile` saves a profile for the message sender. Empty optional fields are
stored as `None`, and the display name must not be empty.

```rust
SetProfile {
    display_name: "Alice".to_string(),
    bio: Some("CosmWasm builder".to_string()),
    website: Some("https://example.com".to_string()),
}
```

`ClearProfile` removes the sender's profile.

```rust
ClearProfile {}
```

## Query Messages

`GetProfile` returns a single profile by address.

```rust
GetProfile {
    address: "alice".to_string(),
}
```

`ListProfiles` returns stored profiles in address order. The query accepts an
optional `start_after` cursor and a bounded `limit`.

```rust
ListProfiles {
    start_after: None,
    limit: Some(10),
}
```
