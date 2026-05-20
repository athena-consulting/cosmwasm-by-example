# Expiring Announcements

This example stores owner-managed announcements that expire at a chosen block
height.

It is useful for learning how a contract can keep public records available while
letting queries distinguish active notices from expired or archived ones. This
demonstrates:

- owner-only execution
- multiple records stored with `cw-storage-plus::Map`
- validation of empty fields and duplicate IDs
- block-height expiry checks with `env.block.height`
- paginated-style list queries with an active-only filter

## Flow

1. Instantiate the contract. The sender becomes the owner.
2. The owner publishes an announcement with an ID, title, body, and expiration
   height.
3. Anyone can query a specific announcement or list announcements.
4. The owner can archive an announcement before or after it expires.
5. Active-only list queries omit archived announcements and announcements whose
   expiration height has passed.
