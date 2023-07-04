## Cross contract Instantiation

This is a example of a CosmWasm contract that implements cross contract instantiation i.e. instantiation another contract from my another contract

---

### Working

#### Instantiate

To understand the working of this example first instantiate the contract using the function `instantiate` in `init.rs`.

```rust
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError>
```

This will deploy the contract and will give contract address which will be used to interact with the contract.

#### Execute

This example contract's Execute endpoint will be called by any user who want to instantiate another contract

In this we have created two different contract `contract-1` and `contract-2`. We will use contract-2 to instantiate contract-1

```rust
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Instantiate {} => try_instantiate(deps,info),
    }
}
```
This `execute` function takes a enum of `ExecuteMsg` which actually contains all the contract function and matches them with the function user is calling. In our case `Instantiate`. Then it calls `try_instantiate` function:

```rust
pub fn try_instantiate(_deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // Creating instantiate message of contract 1 from this contract
    let instantiation_msg = contract_1::msg::InstantiateMsg { count: 1 };

    // Instantiating the contract 1
    let msgs = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
        admin: None,
        code_id: 0,
        msg: to_binary(&instantiation_msg)?,
        funds: info.funds,
        label: "Cross_Contract_instantiation".to_string(),
    });

    let res = Response::new()
        .add_attribute("method", "instantiate_another")
        .add_message(msgs);

    Ok(res)
}
```

It creates a `InstantiateMsg` struct containing the necessary information for the `contract-1` contract instantiation:

- count: The value of count set to 1.

It is then serialized into `binary` using `to_binary` function from `cosmwasm_std`.

It creates a new Response object and adds a `CosmosMsg::Wasm(WasmMsg::Instantiate)` message to it. This message specifies the following details for `contract-1` contract instantiation:

- admin: The admin address for the `contract-1` contract (set to None to have no admin).
- code_id: The code ID of the `contract-`1 contract to instantiate.
- msg: The serialized instantiate_msg binary.
- funds: The funds to be sent along with the instantiation .

Finally, the Response object is returned, which will include the `CosmosMsg::Wasm(WasmMsg::Instantiate)` message that triggers the `contract-1` contract instantiation.

---




