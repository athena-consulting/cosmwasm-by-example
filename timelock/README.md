# Timelock 🔒

**Timelock** is a smart contract that introduces a delay mechanism for executing function calls on other smart contracts. It establishes a predefined minimum time delay before a scheduled operation can be executed.

While Timelock is not strictly a MultiSig/Voting Contract, it aligns closely with the principles of CW3-spec compliant contracts. Instead of immediate execution, addresses can only propose or schedule operations, which then undergo a delay before final execution is allowed.

## Key Features 🌟

- **Instantiation**: Upon deploying the Timelock contract, you can set:
  - A minimum, contract-wide, default time delay.
  - Addresses to act as Administrators and Proposers.

- **Time Delay Mechanism**: 
  - Operations can only be scheduled by proposers if their execution time exceeds the set delay.

- **Administrators**:
  - Handle initial configuration and ensure compatibility with potential target contracts.
  - By default, the contract initiator becomes the administrator if no other addresses are provided.
  - Post configuration, administrators can freeze the Timelock, making it immutable. This action is irrevocable and can render the contract unusable.

- **Proposers**:
  - Schedule operations to be executed after the delay.
  - Ensure that the Timelock contract has necessary permissions on target contracts.
  - Specify executor addresses responsible for the final operation execution on the target contract.
  - If no executors are specified, any address can execute once the time arrives.

- **Note**: Scheduling doesn't guarantee execution. Scheduled operations can be cancelled by the proposer before execution. Thus, choosing proposers is crucial.

## Contract Structures 🛠

### Instantiate
```rust
pub struct InstantiateMsg {
  pub admins: Option<Vec<String>>,
  pub proposers: Vec<String>,
  pub min_delay: Duration,
}
```

### Query 
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

### Execute 
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
