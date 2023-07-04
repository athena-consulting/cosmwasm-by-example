use std::fmt::{Display, Formatter, Result};
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128, Coin};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The NFT contract
    pub cw721_address: Addr,
    /// The token used to pay for NFTs
    pub denom: String,
    /// Marketplace fee collector address
    pub collector_address: Addr,
    /// Marketplace fee
    pub trading_fee_percent: Decimal,
    /// The operator addresses that have access to certain functionality
    pub operators: Vec<Addr>,
    /// Min value for an Auction starting price
    pub min_price: Uint128,
    /// The minimum difference between incremental bids
    pub min_bid_increment: Uint128,
    /// The minimum duration of an auction 
    pub min_duration: u64,
    /// The maximum duration of an auction 
    pub max_duration: u64,
    /// The duration the Auction remains in the Closed state
    pub closed_duration: u64,
    /// The duration an Auction is extended by when a bid is placed in the final minutes
    pub buffer_duration: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub type TokenId = String;

/// Represents a bid (offer) on an auction in the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AuctionBid {
    pub bidder: Addr,
    pub price: Coin,
}

/// Represents an auction on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Auction {
    pub token_id: TokenId,
    pub seller: Addr,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub starting_price: Coin,
    pub reserve_price: Option<Coin>,
    pub funds_recipient: Option<Addr>,
    pub highest_bid: Option<AuctionBid>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum AuctionStatus {
    Pending,
    Open,
    Closed,
    Expired,
}

impl Display for AuctionStatus {
    fn fmt(&self, f: &mut Formatter) -> Result {
       write!(f, "{:?}", self)
    }
}

impl Auction {
    pub fn get_recipient(&self) -> Addr {
        let self_cpy = self.clone();
        self_cpy.funds_recipient.map_or(self_cpy.seller, |a| a)
    }

    pub fn get_auction_status(&self, now: &Timestamp, closed_duration: u64) -> AuctionStatus {
        if now < &self.start_time {
            AuctionStatus::Pending
        } else if now < &self.end_time {
            AuctionStatus::Open
        } else if now < &self.end_time.plus_seconds(closed_duration) {
            AuctionStatus::Closed
        } else {
            AuctionStatus::Expired
        }
    }

    pub fn get_next_bid_min(&self, min_bid_increment: Uint128) -> Uint128 {
        if let Some(_highest_bid) = &self.highest_bid {
            _highest_bid.price.amount + min_bid_increment
        } else {
            self.starting_price.amount
        }
    }

    pub fn is_reserve_price_met(&self) -> bool {
        self.reserve_price.as_ref().map_or(
            false,
            |r| self.highest_bid.as_ref().map_or(false, |h| h.price.amount >= r.amount)
        )
    }
}

/// Primary key for asks
pub type AuctionKey = TokenId;

/// Defines indices for accessing Auctions
pub struct AuctionIndices<'a> {
    pub start_time: MultiIndex<'a, u64, Auction, AuctionKey>,
    pub end_time: MultiIndex<'a, u64, Auction, AuctionKey>,
    pub highest_bid_price: MultiIndex<'a, u128, Auction, AuctionKey>,
    pub seller_end_time: MultiIndex<'a, (String, u64), Auction, AuctionKey>,
    pub highest_bidder_end_time: MultiIndex<'a, (String, u64), Auction, AuctionKey>,
}

impl<'a> IndexList<Auction> for AuctionIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Auction>> + '_> {
        let v: Vec<&dyn Index<Auction>> = vec![
            &self.start_time,
            &self.end_time,
            &self.highest_bid_price,
            &self.seller_end_time,
            &self.highest_bidder_end_time,
        ];
        Box::new(v.into_iter())
    }
}

pub fn auctions<'a>() -> IndexedMap<'a, AuctionKey, Auction, AuctionIndices<'a>> {
    let indexes = AuctionIndices {
        start_time: MultiIndex::new(
            |a: &Auction|  a.start_time.seconds(),
            "auctions",
            "auctions__start_time",
        ),
        end_time: MultiIndex::new(
            |a: &Auction|  a.end_time.seconds(),
            "auctions",
            "auctions__end_time",
        ),
        highest_bid_price: MultiIndex::new(
            |a: &Auction|  a.highest_bid.as_ref().map_or(0, |b| b.price.amount.u128()),
            "auctions",
            "auctions__highest_bid_price"
        ),
        seller_end_time: MultiIndex::new(
            |a: &Auction|  (a.seller.to_string(), a.end_time.seconds()),
            "auctions",
            "auctions__seller_end_time",
        ),
        highest_bidder_end_time: MultiIndex::new(
            |a: &Auction|  (a.highest_bid.as_ref().map_or(String::from(""), |b| b.bidder.to_string()), a.end_time.seconds()),
            "auctions",
            "auctions__highest_bidder_end_time",
        ),
    };
    IndexedMap::new("auctions", indexes)
}
