# Hello World Smart Contract
Smart Contract that implements the query part to query Hello World from the smart contract. 

## Query Message
A "Hello World" string can be queried from the smart contract using this function: 
```rust
// contract.rs

pub fn query_hello_world() -> StdResult<HelloWorldResponse> {
    // Sets the string in the struct to `HelloWorldResponse` and returns it as respomse to query
    let hello_message = HelloWorldResponse {hello_world_message: "Hello World".to_string()};
    Ok(hello_message)
}
```