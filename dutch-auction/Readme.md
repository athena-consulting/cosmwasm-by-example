# Dutch Auction Contract

The Dutch Auction Contract is a smart contract implementation that enables the creation and execution of Dutch Auctions. In a Dutch Auction, the price of an item is initially set high and decreases gradually over time until a buyer is willing to purchase it at the current price. It is commonly used for selling assets like tokens or NFTs.

---

## Features

- Dutch Auction Creation: The contract allows the creation of Dutch Auctions with configurable parameters such as the starting price, price decrement rate, and auction duration.
- Price Calculation: The contract dynamically calculates the current price based on the elapsed time and price decrement rate. This ensures that the price gradually decreases until the auction ends.
- Auction Expiration Handling: The contract verifies if the auction has expired before executing a buy transaction. If the auction has expired, no further purchases can be made.
- Funds Verification: The contract checks if the sent funds are sufficient to purchase the item. If the funds are insufficient, the contract rejects the transaction.
- Automatic Settlement: Upon successful purchase or auction expiration, the contract settles the auction by transferring the item (e.g., tokens or NFTs) from the seller to the buyer.
- Refund Mechanism: If the sent funds exceed the current price, the contract refunds the excess funds to the buyer.
Optional Bid Increments: The contract supports bid increments, allowing the seller to define a minimum amount by which the price can change between consecutive bids.

## Usage

To use the Auction-English contract, you need to:

1. Instantiate the contract by calling the instantiate function. This initializes the auction with the desired parameters:

```rust
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        seller: info.sender,
        starting_price: Uint128::from(msg.starting_price),
        start_at: _env.block.time,
        expires_at: _env.block.time.plus_days(7), // 7 days
        discount_rate: Uint128::from(msg.discount_rate),
        nft_address: msg.nft_address,
        nft_id: Uint128::from(msg.nft_id),
        denom: msg.denom,
    };

    state.save(deps.storage).unwrap();

    Ok(Response::default())
}
```

2. Execute the contract by calling the execute function. This allows a buyer to participate in the auction and purchase the item:

```rust
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyNft {} => {
            let state = State::load(deps.storage).unwrap();
            let mut msgs: Vec<CosmosMsg> = Vec::new();

            // Check if the auction has expired
            if env.block.time >= state.expires_at {
                return Err(ContractError::AuctionExpired {});
            }

            // Calculate the current price based on the discount rate and time elapsed
            let current_price = calculate_current_price(
                state.starting_price,
                state.discount_rate,
                state.start_at.seconds(),
                env.block.time.seconds(),
            );

            // Check if the sent funds are enough to purchase the NFT
            if info.funds.is_empty() || info.funds[0].denom != state.denom {
                return Err(ContractError::InvalidDenomination {});
            }
            let sent_funds = info.funds[0].amount;
            if sent_funds < current_price {
                return Err(ContractError::InsufficientFunds {
                    expected: current_price,
                    actual: sent_funds,
                });
            }

            // Transfer the NFT from the seller to the buyer
            let transfer_nft_msg = TransferNft {
                recipient: info.sender.to_string(),
                token_id: state.nft_id.to_string(),
            };
            msgs.push(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: state.nft_address.to_string(),
                msg: to_binary(&transfer_nft_msg)?,
                funds: info.funds,
            }));

            // Calculate the refund amount and transfer it to the buyer
            let refund = sent_funds - current_price;
            if refund > Uint128::zero() {
                msgs.push(CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: vec![coin(refund.into(), state.denom.clone())],
                }));
            }

            // Remove the contract state after successful execution
            state.remove(deps.storage)?;

            Ok(Response::new()
                .add_attribute("action", "buy")
                .add_messages(msgs))
        }
    }
}

```

4. Query the contract by calling the query function. This allows retrieving information from the contract, such as the current price

```rust
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetPrice {} => {
            let state = State::load(deps.storage).unwrap();
            let current_price = calculate_current_price(
                state.starting_price,
                state.discount_rate,
                state.start_at.seconds(),
                _env.block.time.seconds(),
            );
            Ok(to_binary(&current_price).unwrap())
        }
    }
}
```

For more detailed instructions on deploying and interacting with the Dutch Auction contract, refer to the code comments.