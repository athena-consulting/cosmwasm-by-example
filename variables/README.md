# **Variables in Cosmwasm**

An overview of state and global variables within Cosmwasm and methods to access them.

## **1. Deps, Env, and Info**

These are global structures that provide access to the blockchain's global variables, enabling read/write operations to the blockchain storage.

```rust
// contract.rs

pub fn instantiate(
    /* Deps provides:
       1. Read/Write Storage Access
       2. General Blockchain APIs
       3. Querier to the blockchain (raw data queries) */
    deps: DepsMut,

    /* env grants access to environment information, such as:
       - Block Time/Height
       - Contract address
       - Transaction Info */
    _env: Env,

    /* MessageInfo provides information for authorization, like:
       1. Funds accompanying the message.
       2. The sender (signer) of the message. */
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Initializing the state to be stored on the blockchain. */
    let state = State {
        count: msg.count,
        /* info.sender is the address of the signer of the message. 
           Here, it's stored as the contract owner for access control purposes. */
        owner: info.sender.clone(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Storing the state in deps.storage establishes a space for contract data on the blockchain.
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("count", msg.count.to_string()))
}
```

## **2. State Variables**
State variables are designed to be stored on the blockchain, ensuring data permanence and security.

```rust
// state.rs

pub struct State {
    /* 'count' is a state variable, signifying its storage on the blockchain. */
    pub count: i32,
    pub owner: Addr,
}
