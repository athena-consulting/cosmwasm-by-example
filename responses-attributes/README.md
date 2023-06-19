# Responses and Attributes in Cosmwasm
A response is the final point of a Cosmwasm endpoint (instantiation and execution). It has seperate uses: 
1. It is responsible of returning the metadata when a certain endpoint is called.
A response holds `attributes` which are key-value pairs that are attached when a message is executed that can be queried.
2. It triggers events that can be then queried on a Tendermint or low-level.
3. It can hold messages that the contract should execute when the function is completed.
4. It can return data to the user, which is important in queries.

## Attributes
Attributes are metadata that are used to log information about a certain endpoint to help with filtering and indexing data. The method attribute is normally added to represent an instantiation has been executed on the contract and can be used to filter the instantiation message.
```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}
```

## Messages
Responses can hold messages that the contract executes after all the code inside a function is successfully completed.
These messages are synchronous and the message is part of the transaction itself. If the transaction or message fails to execute, then the whole execution fails and reverts. Similarly to the [Send Tokens example](https://github.com/athena-consulting/cosmwasm-by-example/tree/main/send-tokens), the message could be any CosmosMsg which includes Bank messages (sending native funds), Staking messages or Wasm messages (instantiating and executing).

```rust
Ok(Response::new().add_attribute("action", "increment")
        .add_message(BankMsg::Send { to_address: to.into_string(), amount: vec![Coin{denom, amount}] }))
```
This message in the response is executed by the smart contract after the logic of the function is completed.

## SubMessages
A Submessage can be used instead of a message when a reply is expected as a callback from calling a certain function on another contract or any other message. 
A submessage is asynchronous meaning that even if the message fails on the recipient contract, the whole transaction does not fail but a reply could be sent to handle the error respectively. This is extremely important in IBC use cases where if the packet is rejected, we should not want the whole transaction to fail but only to get a reply of an aknowledgment of failure from the other contract.

```rust
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    // other parse code here...
    
    let msg = Cw20ExecuteMsg::Transfer {
        recipient,
        amount: coin.amount,
    };
    let exec = WasmMsg::Execute {
        contract_addr: coin.address,
        msg: to_binary(&msg).unwrap(),
        funds: vec![],
    };
    let sub_msg = SubMsg::reply_on_error(exec, SEND_TOKEN_ID);
    
    // return submessage with pre-emptive success packet in the data field
    IbcReceiveResponse::new()
        .set_ack(ack_success())
        .add_submessage(sub_msg)
        .add_attributes(attributes)
}
```
