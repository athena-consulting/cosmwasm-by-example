# Read-Write State
Smart contract that explains the basics of reading and writing to the state in a smart contract. 
The state of the smart contract is the allocated storage for the smart contract in the blockchain. 
The state is definted usually in a separate file `state.rs` that imports and uses `Item` and `Map` and other types from `cw_storage_plus`.    

Read more about the specifics of storage in Cosmwasm and Cosmos [here](https://github.com/CosmWasm/cw-storage-plus).

## The State File
```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

// Map is another type of storage that stores key-value pairs in storage.
pub const NAMES: Map<String, String> = Map::new("names");

```
Each constant variable in state is stored as a key value pair in the blockchain and therefor can be read or altered.

## Writing to State
```rust
// contract.rs

pub fn write(deps: DepsMut) -> Result<Response, ContractError> {
        /* This execute endpoint writes a new owner to state. */
        
        // First we need to load the current state from the blockchain from `deps.storage` as mutable.
        let mut state = STATE.load(deps.storage)?;
        state.count = 5;
        
        // Save the new state with the changed variables in storage.
        STATE.save(deps.storage, &state)?;

        // Now let us add a new key-value pair to the `NAMES` map in storage. 
        NAMES.save(deps.storage, "Georges".to_string(), &"Chouchani".to_string())?;
        
        Ok(Response::new().add_attribute("action", "write"))
    }
```
Structs can be stored as an `Item` in storage and key-value pairs are stored as `Map`. 

## Reading from State
### Reading from Structs or constants
```rust
// contract.rs
pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        // Loads the state from storage and checks the count variable.
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
```
The constant is loaded from storage and variables inside the struct (if it is one) can be directly accessed.

### Reading Maps
Maps are usually read by supplying a key and checking if a value exists. 
Keys and values can also be iterated through. 

```rust
pub fn name(deps: Deps, first_name: String) -> StdResult<GetNameResponse> {
        // Loads the NAMES Map from storage for the key `first_name` to get the `last_name`
        // `may_load` returns None if the key does not exist in the map. `load` returns an error.
        let res = NAMES.may_load(deps.storage, first_name)?;
        Ok(GetNameResponse{family_name: res.unwrap()})
    }
```
We load the value of a certain key, it should return None if it does not exist or the value if it does. 

## Future Storage Examples
These storage examples are the simple storage read-write examples but cover more than 90% of use cases. Future contracts will have more complex examples of storage with more complex use cases. 