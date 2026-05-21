# Achievement Badges Contract

This example shows a simple on-chain achievement badge registry.

The contract owner creates badge definitions and manages issuer accounts. The owner or an approved issuer can award a badge to an address. Awarded badges can be queried by badge id, holder address, or holder-and-badge pair.

## Features

- Owner-managed badge definitions.
- Optional issuer accounts for awarding badges.
- One award per holder per badge.
- Per-holder badge limit.
- Badge archiving to stop future awards without deleting history.
- Queries for config, badge definitions, holder badges, and badge ownership.

## Instantiate

```json
{
  "name": "Learning Badges",
  "description": "Badges for completed tutorials",
  "issuers": ["cosmos1issuer..."],
  "max_badges_per_holder": 25
}
```
## Execute

Create a badge:

```json
{
  "create_badge": {
    "badge_id": "first-contract",
    "title": "First Contract",
    "description": "Completed a first CosmWasm contract"
  }
}
```

Award a badge:

```json
{
  "award_badge": {
    "badge_id": "first-contract",
    "recipient": "cosmos1recipient...",
    "note": "Completed the hello-world lesson"
  }
}
```

Archive a badge:

```json
{
  "archive_badge": {
    "badge_id": "first-contract"
  }
}
```

## Query

Query one holder's badges:

```json
{
  "holder_badges": {
    "holder": "cosmos1recipient...",
    "start_after": null,
    "limit": 10
  }
}
```

Check whether a holder has a badge:

```json
{
  "has_badge": {
    "holder": "cosmos1recipient...",
    "badge_id": "first-contract"
  }
}
```
