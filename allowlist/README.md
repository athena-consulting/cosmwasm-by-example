# Allowlist Registry

This example shows a small admin-controlled allowlist contract. It is useful for learning how to:

- set an administrator during instantiation,
- restrict execute messages to the administrator,
- validate account addresses before saving them,
- store address-specific metadata in contract state,
- query whether an address is currently allowlisted.

## Instantiate

```rust
pub struct InstantiateMsg {
    pub admin: Option<String>,
}
```

If `admin` is omitted, the sender that instantiates the contract becomes the administrator.

## Execute

```rust
pub enum ExecuteMsg {
    AddMember { address: String, label: String },
    RemoveMember { address: String },
}
```

Only the administrator can add or remove members.

## Query

```rust
pub enum QueryMsg {
    IsMember { address: String },
}
```

The query returns:

```rust
pub struct MemberResponse {
    pub is_member: bool,
    pub label: Option<String>,
}
```

