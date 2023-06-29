
# Timelock
Timelock is a smart contract designed to delay execute-function calls on other smart contracts with a predetermined minimum time delay. Though it's not strictly a MultiSig/Voting Contract, Timelock follows the footsteps of CW3-spec compliant contracts in the sense that no address can immediately execute, but only propose/schedule an arbitrary operation, before a delayed, final execution can occur.

Instantiating the Timelock contract involves setting up a minimum, contract-wide, default time delay, as well as specifying the addresses to act as Administrators and Proposers.

* The designated minimum time delay for the Timelock contract ensures that operations can only be scheduled by the proposers if their execution time is further in the future than the amount of this delay.


* The administrators are responsible for the initial configuration of the Timelock contract, as well as testing its compatibility with potential target contracts.
  * If the administrator list is left empty, by default, the address by which the Timelock contract is instantiated will be set as an administrator.
  * Once the list of proposers and the minimum time delay of the contract is agreed upon and finalized (upon instantiation or later on by the administrators), the administrators are expected to freeze the Timelock contract to ascertain that no future alterations can be made on the final configuration.
  * Freezing the Timelock contract is irrevocable and may potentially render the contract practically unusable.


* The proposers are in charge of scheduling operations that will pass through the Timelock delay mechanism.
  * A Timelock contract should have the necessary rights on target contracts for scheduled operations to be executed successfully.
  * While scheduling an operation, proposers can specify a list of executor addresses that will be in charge of executing the scheduled operation once the execution time for that particular operation is reached. Executing operations trigger the embedded execute-function call on the target contract as a final step.
  * If the list of executors is left empty by the proposer, any address can execute the scheduled operation once the execution time arrives, by default.

It is important to note that while the Timelock contract is designed to delay execute-function calls, scheduling operations does not guarantee their execution on target contracts per se, considering the fact that a scheduled operation can still be cancelled by the original proposer address before its execution. Therefore, the list of proposers should be carefully contemplated upon before setting up a Timelock contract and freezing its configuration variables.

## Instantiate
```rust
pub struct InstantiateMsg {
  pub admins: Option<Vec<String>>,
  pub proposers: Vec<String>,
  pub min_delay: Duration,
}
```
## Execute
```rust
pub enum ExecuteMsg {
  Schedule {
    target_address: String,
    data: Binary,
    title: String,
    description: String,
    execution_time: Scheduled,
    executors: Option<Vec<String>>,
  },

  Cancel {
    operation_id: Uint64,
  },

  Execute {
    operation_id: Uint64,
  },

  RevokeAdmin {
    admin_address: String,
  },

  AddProposer {
    proposer_address: String,
  },

  RemoveProposer {
    proposer_address: String,
  },

  UpdateMinDelay {
    new_delay: Duration,
  },

  Freeze {},
}
```

## Query
```rust
pub enum QueryMsg {
  GetOperationStatus {
    operation_id: Uint64,
  },

  GetExecutionTime {
    operation_id: Uint64,
  },

  GetAdmins {},

  GetOperations {
    start_after: Option<u64>,
    limit: Option<u32>,
  },

  GetMinDelay {},

  GetProposers {},

  GetExecutors {
    operation_id: Uint64,
  },
}
```
