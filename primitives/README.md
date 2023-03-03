# Primitives
Early introduction to some primitive variables in Cosmwasm and how they work.

## Testing Integers 
Cosmwasm has wrappers around unsigned integers in Rust for better JSON rust encoding and decoding. 

```rust 
pub fn integers(_deps: Deps) -> StdResult<GetIntegerResponse> {
        /* Uint family in cosmwasm are thin wrappers around unsigned integers in Rust that uses strings for JSON encoding and decoding
        Uint64 is the smallest Uint among the group and
        The largest is Uint512
        
        Uints range from 0 to 2**(n-1) where n is the number of bytes for each different primitive type
        i.e Uint256 ranges from 0 to 2**255 */
        
        // verify Max of Uint128 
        // Use Uint::from() to convert from Rust primitive type to Cosmwasm Unsigned integer
        let _ex = Uint64::from(2u64);
        // Can return default value of cosmwasm primitives, in the case of Uint returns 0
        let _def = Uint128::default();
        let max_u128 = Uint128::from(340_282_366_920_938_463_463_374_607_431_768_211_455u128);
        let max_uint128_primitive = Uint128::MAX;
        assert_eq!(max_u128, max_uint128_primitive);

        /* Best practice is to avoid using too large unsigned integers in smart contracts
         for constants or numbers that should not exceed the max for the number. Always check for overflow (later example) */
        Ok(GetIntegerResponse { works: true })
    }
```

## Info.sender
`info.sender` is a global variable that represents infromation on the address signing/sending the transaction.

```rust
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Saves the state of the smart contract from the Instantiate Msg */
    let state = State {
        /* Info.sender is a global function variable that explains who is the signer of a message. */
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        )
}
```

## Addr
`Addr` is a Cosmwasm-std primitive that allows developers to verify that strings represent possible valid addresses.

```rust
pub struct State {
    /* Addr is a cosmwasm-std primitive that helps validate addresses on the Cosmos ecosystem,
     a wrapper for a string that also validates the address in use. */
    pub owner: Addr,
}
```