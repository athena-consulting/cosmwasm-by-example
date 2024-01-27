## **Receive CW20 Tokens in Your Contract**

An example of a CosmWasm contract that implements the [Cw20 Receiver Interface](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md#receiver).

### **How It Works**

#### **1. Instantiation**

To grasp the workings of this example, begin by instantiating the contract using the `instantiate` function located in `init.rs`.

```rust
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
    /* MessageInfo provides authorization data, like:
       1. Funds accompanying the message.
       2. The message's sender (signer). */
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ...
}
```

Upon deployment, this function provides a contract address for sending CW20 tokens. The function takes parameters such as token_symbol and token_contract_address, which are subsequently whitelisted to authorize the token contract to send CW20 to our contract.

#### **2. Execution **
The example contract's execution endpoint is triggered by the Cw20 contract, not the user.
```rust
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    ...
}
```

For the CW20 contract to perform this, the user must invoke the Send{contract, amount, msg} method on the Cw20 contract.

The function execute_receive outlined below is called upon receiving the CW20 token:

```rust
pub fn execute_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    ...
}
```

The execute_do_something function, invoked as part of the above method, outlines the required execution logic upon CW20 token receipt:

```rust
pub fn execute_do_something(
    _deps: DepsMut,
    _user_wallet: &Addr,
    _cw20_contract_addr: &Addr,
    _balance: Balance,
) -> Result<Response, ContractError> {
    ...
}
```

#### **3. Query**
The example's query endpoint retrieves the contract's admin information.

```rust
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    ...
}

pub fn get_admin(deps: Deps) -> Result<QueryResponse, StdError> {
    ...
}
```

The function get_admin fetches the contract's global state CONFIG from state.rs, yielding details like the admin address and the whitelisted contract associated with its token symbol.

```rust
// Config structure with contract admin
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub cw20_wl: Vec<(String, Addr)>,
}
```

