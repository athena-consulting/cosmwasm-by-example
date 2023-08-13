# Hello World Smart Contract 🌍

This smart contract provides an implementation for querying a "Hello World" message.

## Query Message 📩

Retrieve the "Hello World" string from the smart contract using the following function:

```rust
// contract.rs

pub fn query_hello_world() -> StdResult<HelloWorldResponse> {
    // Sets the string in the struct to `HelloWorldResponse` and returns it as a response to the query
    let hello_message = HelloWorldResponse {hello_world_message: "Hello World".to_string()};
    Ok(hello_message)
}
