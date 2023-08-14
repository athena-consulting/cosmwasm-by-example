# **Smart Contract Initialization**

This document elucidates the instantiation process of a smart contract. It discusses the `InstantiateMsg` structure passed during contract creation and the `instantiate` function that runs upon contract execution.

## **1. InstantiateMsg: Initialization Message**

`InstantiateMsg` houses the variables provided when the contract gets instantiated. Ensure sensitive data prone to replay attacks isn't incorporated.

```rust
/// msg.rs

/// Variables for contract instantiation.
/// Exclude variables susceptible to replay attacks or secret data.
pub struct InstantiateMsg {
    pub sent_message: String,
}
```

## **2. Instantiate: Contract Execution

On contract creation, the instantiate function activates. It sets the initial state of the smart contract, conducts necessary checks, and can function akin to an execute message.

```rust
/// contract.rs

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Initialize contract's state.
    let state = State {
        global_var: msg.sent_message,
    };
    // Save state to the blockchain.
    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("instantiated", "true"))
}

```
