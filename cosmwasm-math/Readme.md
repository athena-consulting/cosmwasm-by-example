# Cosmwasm Maths Operations

This is a  example of a CosmWasm contract that implements simple maths operations like addition, substraction, multiplication, division, modulo and exponential

---

## Math Operations

```rust
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
    ExecuteMsg::Operations { a, b } => execute_operations(deps,a,b),
}
}
```

This `execute` function takes a enum of `ExecuteMsg` which actually contains all the contract function and matches them with the function user is calling. In our case `Operations`. Then it calls `execute_operations` function:

```rust
pub fn execute_operations(deps: DepsMut, a: u128, b: u128) -> Result<Response, ContractError> {
        // Checking if numbers are not zero
        if a == 0 && b == 0 {
            return Err(ContractError::CanNotBeZero());
        }

        // Addition
        let addition_result = a + b;

        // Subtraction
        let subtraction_result = a - b;

        // Multiplication
        let multiplication_result = a * b;

        // Division
        let division_result = a / b;

        // Modulo
        let modulo_result = a % b;

        // Exponentiation
        let exponent: u32 = 3;
        let exponentiation_result: u128 = a.pow(exponent);

        // Create the response
        let response = OperationsResponse {
            addition_result,
            subtraction_result,
            multiplication_result,
            division_result,
            modulo_result,
            exponentiation_result,
        };

        // Fetching the state
        RESULT.load(deps.storage).unwrap();

        // Update the state
        RESULT.save(deps.storage, &response).unwrap();

        let res = Response::new().add_attributes(vec![
            ("action", "operations"),
            ("a", &a.to_string()),
            ("b", &b.to_string()),
            ("addition_res", &addition_result.to_string()),
            ("substraction_res", &subtraction_result.to_string()),
            ("multiplicationn_res", &multiplication_result.to_string()),
            ("division_res", &division_result.to_string()),
            ("modulo_res", &modulo_result.to_string()),
            ("exponential_res", &exponentiation_result.to_string()),
        ]);

        Ok(res)
    }
```

This function takes two parameters a and b for mathmatical operations and store the result in `RESULT` global state variable stored in `state.rs` :

```rust
pub const RESULT: Item<OperationsResponse> = Item::new("result");
```

***NOTE  We are using `Item` here for storage if we want better storage options then we can use `MAP` from `cw_storage_plus` which store values in key-value pairs.
We can query the result using next step.

### Query

This example query endpoint is basically getting the result  of mathmatical operations  .

```rust
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {
        QueryMsg::GetResponse {} => get_response(deps),
    }
}
```
This `query` function takes a enum of `QueryMsg` which actually contains all the contract query function and matches them with the function user is calling. In our case `GetResponse`. Then it calls `get_response` function:

```rust
pub fn get_response(deps: Deps) -> Result<QueryResponse, StdError> {
    let result = RESULT.load(deps.storage)?;

    to_binary(&result)
}
```
 This function return the result of our mathmatical operation.

---

