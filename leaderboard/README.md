# Leaderboard

This example shows a simple leaderboard contract for tracking each player's best score.

The contract stores one score per player address. A player can submit a score for
themselves, and the contract only replaces an existing entry when the new score is
higher. The owner can submit or remove scores for any player and can update the
display-name and metadata limits.

## Messages

- `SubmitScore`: stores a player's best score with a display name and optional metadata.
- `RemoveScore`: removes a score. Players can remove their own entry; the owner can remove any entry.
- `UpdateConfig`: lets the owner transfer ownership or adjust validation limits.

## Queries

- `Config`: returns the owner and validation limits.
- `Score`: returns one player's current best score.
- `TopScores`: returns top scores sorted from highest to lowest.
- `PlayerCount`: returns the number of stored player entries.

## Example flow

1. Instantiate the contract with optional display-name and metadata limits.
2. Players call `SubmitScore` with their display name, score, and optional metadata.
3. Query `TopScores` to show the leaderboard.
4. A player can improve their score by submitting a higher one.
