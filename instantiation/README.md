# Instantiation
The instantiation message and creating a contract.

## InstantiateMsg
The message that is passed to the instantiate function at contract creation.

```rust
/// msg.rs

/*
variables to be passed to contract at instantiation.
secret variables or any variable that is open to a replay attack should not be 
part of the InstantiateMsg 
*/
pub struct InstantiateMsg {
    pub sent_message: String
}
```
## Instantiate
The function that is executed when a contract is executed.
```rust
/// contract.rs
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Code is executed at contract creation
    Used to set initial (default) state variables of the smart contract
    Can perform checks and acts like an execute message.
     */
    let state = State {
        global_var: msg.sent_message
    };
    // Pushes the data to blockchain storage
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
    .add_attribute("instantiated", "true"))
}
```