# English-Auction Contract

This repository contains the implementation of the English-Auction contract, which is designed to facilitate English auctions for NFTs (Non-Fungible Tokens) on the CosmWasm platform. The contract is built using the CosmWasm framework and relies on the `royality-cw721` contract for NFT functionality.

---

## Features

The Auction-English contract provides the following features:

- Conducting English auctions for NFTs: The contract enables the creation and management of English auctions, where participants can place bids on NFTs, and the highest bidder wins the auction.

- Automatic bid increments: The contract allows for automatic bid increments, where each new bid must be higher than the previous bid by a specified increment amount.

- Time-limited auctions: The contract supports setting a start time and a duration for the auction, ensuring that bidding is only allowed within the specified timeframe.

- Pausable functionality: The contract includes pausable functionality provided by the `royality-cw721` contract, allowing the auction to be paused if necessary.

- Royalty distribution: The `royality-cw721` contract integration enables the automatic distribution of royalties to the original creator or rights holder whenever an NFT is sold or transferred.

## Usage

To use the Auction-English contract, you need to:

1. Deploy the Auction-English contract: Compile and deploy the contract to your desired blockchain network using the CosmWasm framework.

2. Mint NFTs: Use the `royality-cw721` contract or another compatible NFT contract to mint the NFTs you want to auction.

3. Create an auction: Invoke the `execute_set_auction` function of the Auction-English contract, providing the necessary parameters such as the NFT ID, starting bid, bid increment, auction duration, and any additional settings.

```rust
pub fn execute_set_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    auction: Auction,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    
    let config = CONFIG.load(deps.storage)?;
    validate_auction_times(&auction, &config, &env.block.time)?;
    
    price_validate(&auction.starting_price, &config)?;
    if let Some(_reserve_price) = &auction.reserve_price {
        price_validate(&_reserve_price, &config)?;
        if _reserve_price.amount < auction.starting_price.amount {
            return Err(ContractError::InvalidReservePrice(_reserve_price.amount, auction.starting_price.amount));
        }
    }

    only_owner(deps.as_ref(), &info, &config.cw721_address, &auction.token_id)?;

    let existing_auction = auctions().may_load(deps.storage, auction.token_id.clone())?;
    if let Some(_existing_auction) = existing_auction {
        return Err(ContractError::AlreadyExists(auction.token_id.clone()));
    }

    auctions().save(deps.storage, auction.token_id.clone(), &auction)?;

    let mut response = Response::new();

    transfer_nft(&auction.token_id, &env.contract.address, &config.cw721_address, &mut response)?;

    let event = Event::new("set-auction")
        .add_attribute("collection", config.cw721_address.to_string())
        .add_attribute("token_id", auction.token_id.to_string())
        .add_attribute("seller", auction.seller)
        .add_attribute("start_time", auction.start_time.to_string())
        .add_attribute("end_time", auction.end_time.to_string())
        .add_attribute("starting_price", auction.starting_price.to_string());

    Ok(response.add_event(event))
}

```

4. Participate in the auction: Bidders can place bids on the auctioned NFT by invoking the `execute_set_auction_bid` function and specifying the bid amount.

```rust
pub fn execute_set_auction_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
    auction_bid: AuctionBid,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    let config = CONFIG.load(deps.storage)?; 

    // Validate auction exists, and is open
    let mut auction = auctions().load(deps.storage, token_id.clone())?;
    let auction_status = auction.get_auction_status(&env.block.time, config.closed_duration);
    match &auction_status {
        AuctionStatus::Open => {},
        _ => return Err(ContractError::InvalidStatus(auction_status.to_string())),
    }

    // Validate bid is higher than the minimum viable bid
    if auction_bid.price.amount < auction.get_next_bid_min(config.min_bid_increment) {
        return Err(ContractError::BidTooLow {});
    }
    
    // If previous bid exists, refund it
    if let Some(prev_highest_bid) = &auction.highest_bid {
        transfer_token(
            prev_highest_bid.price.clone(),
            prev_highest_bid.bidder.to_string(),
            "refund-auction-bidder",
            &mut response,
        )?;
    }

    price_validate(&auction_bid.price, &config)?;
    let payment_amount = must_pay(&info, &config.denom)?;
    if auction_bid.price.amount != payment_amount  {
        return Err(ContractError::IncorrectBidPayment(auction_bid.price.amount, payment_amount));
    }

    auction.highest_bid = Some(auction_bid.clone());
    
    // If auction end time is within buffer_duration, then update the end time
    let new_auction_end_time = env.block.time.plus_seconds(config.buffer_duration);
    if new_auction_end_time > auction.end_time {
        auction.end_time = new_auction_end_time;
    }
    
    auctions().save(deps.storage, auction.token_id.clone(), &auction)?;

    let event = Event::new("set-auction-bid")
        .add_attribute("token_id", &token_id.to_string())
        .add_attribute("bidder", &auction_bid.bidder)
        .add_attribute("price", &auction_bid.price.to_string());
    response.events.push(event);

    Ok(response)
}
```

5. End the auction: Once the auction duration has elapsed, the contract will automatically determine the highest bidder and transfer the NFT to the winner. The funds from the auction will be distributed to the NFT owner and royalty recipients according to the predefined rules.

For more detailed instructions on deploying and interacting with the Auction-English contract, refer to the code comments.

---