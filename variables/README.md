# Variables
Introduction to state and global variables in Cosmwasm and how to access them.

## Deps, Env and Info 
Global structures that allow access to global variables on the blockchain and reading/writing data to blockchain storage. 

```rust
pub fn instantiate(
    /* Deps allows to access:
    1. Read/Write Storage Access
    2. General Blockchain APIs
    3. The Querier to the blockchain (raw data queries) */
    deps: DepsMut,
    /* env gives access to global variables which represent environment information.
    For exaample: 
    - Block Time/Height
    - contract address
    - Transaction Info */
    _env: Env,
    /* Message Info gives access to information used for authorization.
    1. Funds sent with the message.
    2. The message sender (signer). */
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Instantiating the state that will be stored to the blockchain */
    let state = State {
        count: msg.count,
        /* info.sender is the address of the signer of the message and is stored in this instance in storage as the owner of the contract
        (which is used for access control to smart contract functions) */
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Save the stete in deps.storage which creates a storage for contract data on the blockchain. 
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("count", msg.count.to_string()))
}
```

## State Variables
State variables are variables that are meant to be stored in the blockchain. 

```rust
pub struct State {
    /* Count is a state variable which means that the data will be stored on the blockchain*/
    pub count: i32,
    pub owner: Addr,
}
```
