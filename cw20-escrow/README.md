# CW20 Escrow

This is an escrow meta-contract that allows multiple users to
create independent escrows. Each escrow has a sender, recipient,
and arbiter. It also has a unique id (for future calls to reference it)
and an optional timeout.

The basic function is the sender creates an escrow with funds.
The arbiter may at any time decide to release the funds to either
the intended recipient or the original sender (but no one else),
and if it passes with optional timeout, anyone can refund the locked
tokens to the original sender.

## Arbiter 
An Arbiter could either be a contract or an address responsible of releasing funds 
back to the sender or to the reciever. 
For example, an oracle could be set that tracks the price of Bitcoin, if the price of
Bitcoin goes over $30K before the end of the month, the price goes to the receiver end of
the bet. Else it goes back to the sender. 

## Recieving CW20 Tokens 
As explained in [Receiving Cw20](https://www.github.com/athena-consulting/cosmwasm-by-example/receiving-cw20), a contract
should add a `receive` function in order to be able to receive and aknowledge CW20 tokens and call a function. 
Since escrows could hold CW20 tokens, it is important to add them as well.

```rust

pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender,
        amount: wrapper.amount,
    });
    let api = deps.api;
    match msg {
        ReceiveMsg::Create(msg) => {
            execute_create(deps, msg, balance, &api.addr_validate(&wrapper.sender)?)
        }
        ReceiveMsg::TopUp { id } => execute_top_up(deps, id, balance),
    }
}

```

## Top Up 
If for some reason there is a need to top up the funds in the escrow, the escrow can always
be top up by anyone and not only the sender. 

```rust

pub fn execute_top_up(
    deps: DepsMut,
    id: String,
    balance: Balance,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // this fails is no escrow there
    let mut escrow = ESCROWS.load(deps.storage, &id)?;

    if let Balance::Cw20(token) = &balance {
        // ensure the token is on the whitelist
        if !escrow.cw20_whitelist.iter().any(|t| t == &token.address) {
            return Err(ContractError::NotInWhitelist {});
        }
    };

    escrow.balance.add_tokens(balance);

    // and save
    ESCROWS.save(deps.storage, &id, &escrow)?;

    let res = Response::new().add_attributes(vec![("action", "top_up"), ("id", id.as_str())]);
    Ok(res)
}
```

# Credits
Credits to Ethan Frey for writing an example `cw20-escrow` Contract.





