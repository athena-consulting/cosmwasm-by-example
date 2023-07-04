## RECIEVE CW20 TOKENS IN YOUR CONTRACT

This is a  example of a CosmWasm contract that implements the [Cw20 Receiver Interface](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md#receiver)

---

### Working

#### Instantiate

To understand the working of this example first instantiate the contract using the function `instantiate` in `init.rs`.

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
    let admin = info.sender.to_string();

    // contract address with denom that will be whitelisted to send toke to this address
    let cw20_whitelist: Vec<(String, Addr)> = vec![(
        msg.token_symbol,
        msg.token_contract_address,
    )];

    // Save the stete in deps.storage which creates a storage for contract data on the blockchain.
    CONFIG.save(
        deps.storage,
        &Config {
            admin: deps.api.addr_validate(&admin).unwrap(),
            cw20_wl: cw20_whitelist,
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin))
}
```

This will deploy the contract and will give contract address which will be used to send cw20 token to.
The following params of `InstantiateMsg` are :

- `token_symbol`: The symbol of the token
- `token_contract_address`: THe token address 

These above symbol and address will be whitelisted so can token contract can send cw20 to our contract

#### Execute

This example contract's Execute endpoint will be called directly by the Cw20 contract itself not by the user

```rust
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // cw20 receive wrapper
        ExecuteMsg::Receive(receive_msg) => execute_receive(deps, env, info, receive_msg),
    }
}
```

To get the Cw20 contract to do this, the user will need to call the `Send{contract, amount, msg}` execute on the Cw20 contract,
- Where `contract` is the Address of this contract
- Where `amount` is the amount of Cw20 tokens to send to this contract
- Where `msg` is `in binary` the ReceiveMsg of our contract

The msg will be the function we want to call from our contract when the cw2o token will be recieved :

```rust
pub fn execute_receive(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        wrapper: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        // Message included in Send{contract, amount, **msg**} execute on the cw20 contract
        let msg: ReceiveMsg = from_binary(&wrapper.msg).unwrap();
    
        // Address that executed the "Send" on the cw20 contract
        let user_wallet = deps.api.addr_validate(&wrapper.sender).unwrap();
    
        // Constructing cw20 balance
        let balance = Balance::Cw20(Cw20CoinVerified {
            // cw20 contract this message was sent from
            address: info.sender.clone(),
            // Send{contract, **amount**, msg}
            amount: wrapper.amount,
        });
    
        // Load config for whitelist check
        let config = CONFIG.load(deps.storage)?;
    
        // Check constructed cw20 balance , returns contract error if not
        is_balance_whitelisted(&balance, &config)?;
    
        match msg {
            // Message included in the "Send{contract, amount, **msg**}" call on the cw20 contract,
            ReceiveMsg::AnExecuteMsg {} => {
                execute_do_something(deps, &user_wallet, &info.sender, balance)
            }
        }
    }
```

In above snippet we want to call `AnExecuteMsg` when our contract recieves cw20 token which will call `execute_do_something` which will contai ur execution logic of what should happen if we recieve cw20 token.

```rust
 pub fn execute_do_something(
        _deps: DepsMut,
        _user_wallet: &Addr,
        _cw20_contract_addr: &Addr,
        _balance: Balance,
    ) -> Result<Response, ContractError> {
        // insert your execution logic here
    
        Ok(Response::default())
    }
```

#### Query

This example query endpoint is basically getting the admin info of your contract .

```rust
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {
        QueryMsg::GetAdmin {} => get_admin(deps),
    }
}
```
```rust
  pub fn get_admin(deps: Deps) -> Result<QueryResponse, StdError> {
        let config = CONFIG.load(deps.storage)?;
    
        let admin = config.admin.to_string();
    
        to_binary(&AdminResponse { admin })
    }
```

 The above function will basically fetch the contrcat global state `CONFIG` which is defined in `state.rs` and return the things stored under it. In our case admin address and the whitelised contract with its token symbol which is instantiated during instantiation of contract.

 ```rust
 // Config with contract admin
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub cw20_wl: Vec<(String, Addr)>,
}
 ``` 

---
