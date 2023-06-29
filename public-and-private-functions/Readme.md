## COSMWASM MATHS OPERATIONS

This is a  example of a CosmWasm contract that helps to understand public and private functions in cosmwasm

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

```rust
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
    ExecuteMsg::PublicFunction {param} => public_function(deps,env,info,param),
}
}
```
This `execute` function takes a enum of `ExecuteMsg` which actually contains all the contract function and matches them with the function user is calling. In our case `PublicFunction`. Then it calls `public_function` function:

```rust
pub fn public_function(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    param: String,
) -> Result<Response, ContractError> {
    // Perform the desired logic
    let result = do_something(param);

    // Return a response
    let response = Response::new().add_attribute("result", result);

    Ok(response)
}
```

we have a public function called `public_function` that takes a param parameter of type String. This function can be called externally. Inside the public function, we invoke a private function called `do_something`, which performs some internal logic and returns a string result. The public function then constructs a response, including an attribute with the result.

The `do_something `function is a private helper function that can only be called within the contract. It performs some internal logic and returns a result.

```rust
fn do_something(param: String) -> String {
    // Perform some internal logic
    let result = format!("Doing something with param: {}", param);

    result
}
```

---