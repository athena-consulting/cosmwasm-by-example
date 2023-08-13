# Counter
Counter default smart contract that allows incrementing and reseting a counter.

## Changing State Variable
Increment function changes the count state variable and pushes the state to blockchain storage.

```rust
// contract.rs

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }
```

## Reading from State
Query functions load the state from the blockchain and outputs a state variable (or an alteration of it).

```rust
// contract.rs

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
```