# Project Status Registry

Project Status Registry is a small CosmWasm contract that records project milestones and their current review status. It is intended as a simple example of owner-managed records with public queries.

## What It Teaches

- Saving contract configuration with `Item`
- Saving multiple records with `Map`
- Restricting write actions to a validated owner
- Querying a single record or a paginated list of records
- Emitting useful response attributes from execute messages

## Contract Flow

1. Instantiate the contract with an optional owner. If no owner is supplied, the sender becomes the owner.
2. The owner adds project records with an id, title, status, and note.
3. The owner updates a project's status as work moves through review.
4. Anyone can query a project or list the stored projects.

## Messages

### Instantiate

```rust
pub struct InstantiateMsg {
    pub owner: Option<String>,
}
```

### Execute

```rust
pub enum ExecuteMsg {
    AddProject {
        id: String,
        title: String,
        status: ProjectStatus,
        note: String,
    },
    UpdateStatus {
        id: String,
        status: ProjectStatus,
        note: String,
    },
    TransferOwnership {
        new_owner: String,
    },
}
```

### Query

```rust
pub enum QueryMsg {
    Config {},
    Project { id: String },
    ListProjects {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}
```

## Example Use Cases

- Track open-source grant milestones.
- Publish a transparent status log for community projects.
- Keep lightweight on-chain evidence of who last updated a project record.
