# Hash Notary

Hash Notary is a simple CosmWasm contract for timestamping document or artifact hashes without storing private file contents on chain.

Users register a SHA-256 digest with a short label and optional memo. The contract stores the normalized digest, the sender, the block timestamp, and an active flag. The original proof remains queryable even if the owner later revokes it.

## Messages

### Instantiate

```json
{}
```

### Register

```json
{
  "register": {
    "digest": "b94d27b9934d3e08a52e52d7da7dabfade035a3e5094b9f9f1795f7c4f3819d0",
    "label": "service-agreement.pdf",
    "memo": "Signed PDF hash"
  }
}
```

### Update Memo

```json
{
  "update_memo": {
    "id": 1,
    "memo": "Updated internal reference"
  }
}
```

### Revoke

```json
{
  "revoke": {
    "id": 1
  }
}
```

## Queries

### Get Record

```json
{
  "get_record": {
    "id": 1
  }
}
```

### Get Record By Digest

```json
{
  "get_record_by_digest": {
    "digest": "0xb94d27b9934d3e08a52e52d7da7dabfade035a3e5094b9f9f1795f7c4f3819d0"
  }
}
```

### List Records

```json
{
  "list_records": {
    "start_after": 10,
    "limit": 20
  }
}
```

## Validation

- Digests must be SHA-256 hex strings, with or without a `0x` prefix.
- Labels must be non-empty and at most 80 characters.
- Memos are optional and at most 240 characters.
- Only the record owner can update a memo or revoke the record.

## Tests

```bash
cargo test --manifest-path hash-notary/Cargo.toml
```
