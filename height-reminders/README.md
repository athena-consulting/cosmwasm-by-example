# Height Reminders

This example stores simple block-height reminders.

A user creates a reminder with a unique ID, a due block height, and a short note.
The creator can cancel it before it is due. Once the chain reaches the due
height, anyone can mark it as complete. This demonstrates:

- storing multiple records with `cw-storage-plus::Map`
- validating message data during execution
- checking `env.block.height`
- enforcing creator-only cancellation
- querying individual reminders and all known reminder IDs

## Flow

1. Instantiate the contract.
2. Call `CreateReminder` with an ID, due height, and note.
3. Query the reminder by ID or list all IDs.
4. Before the due height, the creator can cancel the reminder.
5. At or after the due height, anyone can complete it.
