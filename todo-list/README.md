# Todo List

This example shows how to build a per-user todo list in CosmWasm.

Each account owns its own task sequence. A user can create tasks, mark them complete, reopen them, delete them, and query their tasks with pagination. The contract owner can update the maximum number of open tasks each user may keep.

## What this example teaches

- Storing global configuration with `Item`
- Tracking per-user counters with `Map`
- Using composite map keys like `(owner, task_id)`
- Limiting active tasks per user
- Updating stored records in place
- Paginating records under a single owner prefix

## Messages

### Instantiate

```json
{
  "max_open_tasks": 25
}
```

If omitted, `max_open_tasks` defaults to `50`.

### Add Task

```json
{
  "add_task": {
    "title": "Read CosmWasm docs",
    "description": "Focus on storage maps and query pagination"
  }
}
```

### Complete Task

```json
{
  "complete_task": {
    "id": 1
  }
}
```

### Reopen Task

```json
{
  "reopen_task": {
    "id": 1
  }
}
```

### Delete Task

```json
{
  "delete_task": {
    "id": 1
  }
}
```

### Update Config

```json
{
  "update_config": {
    "max_open_tasks": 100,
    "new_owner": "cosmos1..."
  }
}
```

Only the contract owner can update configuration.

## Queries

### Config

```json
{ "config": {} }
```

### Task

```json
{
  "task": {
    "owner": "cosmos1...",
    "id": 1
  }
}
```

### Tasks

```json
{
  "tasks": {
    "owner": "cosmos1...",
    "start_after": 1,
    "limit": 10,
    "include_completed": false
  }
}
```

Set `include_completed` to `true` to include finished tasks in the page.
