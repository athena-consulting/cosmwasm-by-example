use crate::error::ContractError;
use crate::state::{
    Config, TokenId, Auction
};
use cosmwasm_std::{
    to_binary, Addr, Api, StdResult, Timestamp, WasmMsg, Order, Deps,
    Event, Coin, coin, Uint128, Response, MessageInfo, BankMsg, SubMsg, Decimal
};
use royality_cw721::msg::{CollectionInfoResponse, QueryMsg as Royality721QueryMsg};
use cw721::{Cw721ExecuteMsg};
use cw721_base::helpers::Cw721Contract;

pub fn map_validate(api: &dyn Api, addresses: &[String]) -> StdResult<Vec<Addr>> {
    addresses
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}

pub fn option_bool_to_order(descending: Option<bool>) -> Order {
     match descending {
        Some(_descending) => if _descending { Order::Descending } else { Order::Ascending },
        _ => Order::Ascending
    }
}

/// Transfers funds and NFT, updates bid
pub fn finalize_sale(
    deps: Deps,
    bidder: &Addr,
    token_id: &TokenId,
    payment_amount: Uint128,
    payment_recipient: &Addr,
    config: &Config,
    res: &mut Response,
) -> StdResult<()> {
    payout(deps, payment_amount, payment_recipient, &config, res)?;

    transfer_nft(&token_id, bidder, &config.cw721_address, res)?;

    let event = Event::new("finalize-sale")
        .add_attribute("collection", config.cw721_address.to_string())
        .add_attribute("buyer", bidder.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("payment_amount", payment_amount.to_string())
        .add_attribute("payment_recipient", payment_recipient.to_string());
    res.events.push(event);

    Ok(())
}

/// Payout a bid
fn payout(
    deps: Deps,
    payment_amount: Uint128,
    payment_recipient: &Addr,
    config: &Config,
    response: &mut Response,
) -> StdResult<()> {
    let cw721_address = config.cw721_address.to_string();

    // Charge market fee
    let market_fee = payment_amount * config.trading_fee_percent / Uint128::from(100u128);
    if market_fee > Uint128::zero() {
        transfer_token(
            coin(market_fee.u128(), &config.denom),
            config.collector_address.to_string(),
            "payout-market",
            response
        )?;
    }

    // Query royalties
    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(&cw721_address, &Royality721QueryMsg::CollectionInfo {})?;

    // Charge royalties if they exist
    let royalties = match &collection_info.royalty_info {
        Some(royalty) => Some((payment_amount * royalty.share, &royalty.payment_address)),
        None => None
    };
    if let Some(_royalties) = &royalties {
        if _royalties.0 > Uint128::zero() {
            transfer_token(
                coin(_royalties.0.u128(), &config.denom),
                _royalties.1.to_string(),
                "payout-royalty",
                response
            )?;
        }
    };

    // Pay seller
    let mut seller_amount = payment_amount - market_fee;
    if let Some(_royalties) = &royalties {
        seller_amount -= _royalties.0;
    };

    transfer_token(
        coin(seller_amount.u128(), &config.denom),
        payment_recipient.to_string(),
        "payout-seller",
        response
    )?;

    Ok(())
}

// Validate Bid or Ask price
pub fn price_validate(price: &Coin, config: &Config) -> Result<(), ContractError> {
    if
        price.amount.is_zero() ||
        price.denom != config.denom ||
        price.amount < config.min_price
    {
        return Err(ContractError::InvalidPrice {});
    }

    Ok(())
}

/// Checks to enforce only NFT owner can call
pub fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<(), ContractError> {
    let res = Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::Unauthorized(String::from("only the owner can call this function")));
    }
    Ok(())
}

/// Checks to enforce only Ask seller can call
pub fn only_seller(
    info: &MessageInfo,
    seller: &Addr,
) -> Result<(), ContractError> {
    if &info.sender != seller {
        return Err(ContractError::Unauthorized(String::from("only the seller can call this function")));
    }
    Ok(())
}

/// Checks to enforce only privileged operators
pub fn only_operator(info: &MessageInfo, config: &Config) -> Result<Addr, ContractError> {
    if !config
        .operators
        .iter()
        .any(|a| a.as_ref() == info.sender.as_ref())
    {
        return Err(ContractError::Unauthorized(String::from("only an operator can call this function")));
    }

    Ok(info.sender.clone())
}

pub fn transfer_nft(token_id: &TokenId, recipient: &Addr, collection: &Addr, response: &mut Response,) -> StdResult<()> {
    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: token_id.to_string(),
        recipient: recipient.to_string(),
    };

    let exec_cw721_transfer = SubMsg::new(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    });
    response.messages.push(exec_cw721_transfer);

    let event = Event::new("transfer-nft")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("recipient", recipient.to_string());
    response.events.push(event);
    
    Ok(())
}

pub fn transfer_token(coin_send: Coin, recipient: String, event_label: &str, response: &mut Response) -> StdResult<()> {
    let token_transfer_msg = BankMsg::Send {
        to_address: recipient.clone(),
        amount: vec![coin_send.clone()]
    };
    response.messages.push(SubMsg::new(token_transfer_msg));

    let event = Event::new(event_label)
        .add_attribute("coin", coin_send.to_string())
        .add_attribute("recipient", recipient.to_string());
    response.events.push(event);

    Ok(())
}

pub fn validate_auction_times(auction: &Auction, config: &Config, now: &Timestamp) -> Result<(), ContractError> {
    if &auction.start_time <= now {
        return Err(ContractError::InvalidStartEndTime(String::from("start time must be in the future")));
    }
    if &auction.start_time.plus_seconds(config.min_duration) > &auction.end_time {
        return Err(ContractError::InvalidStartEndTime(String::from("duration is below minimum")));
    }
    if &auction.start_time.plus_seconds(config.max_duration) < &auction.end_time {
        return Err(ContractError::InvalidStartEndTime(String::from("duration is above maximum")));
    }
    Ok(())
}

pub fn validate_config(config: &Config) -> Result<(), ContractError> {
    if config.trading_fee_percent > Decimal::percent(10000) {
        return Err(ContractError::InvalidConfig(String::from("trading_fee_percent must be less than or equal to 100")));
    }
    if config.operators.is_empty() {
        return Err(ContractError::InvalidConfig(String::from("operators must be non-empty")));
    }
    if config.min_price.is_zero() {
        return Err(ContractError::InvalidConfig(String::from("min_price must be greater than zero")));
    }
    if config.min_bid_increment.is_zero() {
        return Err(ContractError::InvalidConfig(String::from("min_bid_increment must be greater than zero")));
    }
    if config.min_duration == 0 {
        return Err(ContractError::InvalidConfig(String::from("min_duration must be greater than zero")));
    }
    if config.max_duration == 0 {
        return Err(ContractError::InvalidConfig(String::from("max_duration must be greater than zero")));
    }
    if config.min_duration > config.max_duration {
        return Err(ContractError::InvalidConfig(String::from("max_duration must be greater than or equal to min_duration")));
    }
    if config.closed_duration == 0 {
        return Err(ContractError::InvalidConfig(String::from("closed_duration must be greater than zero")));
    }
    Ok(())
}