use crate::state::{TokenId, Config, Auction, AuctionStatus};
use cosmwasm_std::{Coin, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The NFT contract
    pub cw721_address: String,
    /// The token used to pay for NFTs
    pub denom: String,
    /// The address collecting marketplace fees
    pub collector_address: String,
    /// Fair Burn fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Operators are entites that are responsible for maintaining the active state of Asks.
    /// They listen to NFT transfer events, and update the active state of Asks.
    pub operators: Vec<String>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update the contract parameters
    UpdateConfig {
        collector_address: Option<String>,
        trading_fee_bps: Option<u64>,
        operators: Option<Vec<String>>,
        min_price: Option<Uint128>,
        min_bid_increment: Option<Uint128>,
        min_duration: Option<u64>,
        max_duration: Option<u64>,
        closed_duration: Option<u64>,
        buffer_duration: Option<u64>,
    },
    /// Create an auction for a specified token
    SetAuction {
        token_id: TokenId,
        start_time: Timestamp,
        end_time: Timestamp,
        starting_price: Coin,
        reserve_price: Option<Coin>,
        funds_recipient: Option<String>,
    },
    /// Place a bid on an existing auction
    SetAuctionBid {
        token_id: TokenId,
        price: Coin,
    },
    /// Sellers can close a previously created auction that has
    /// not met the reserve price
    CloseAuction {
        token_id: TokenId,
        accept_highest_bid: bool,
    },
    /// Anyone can finalize an auction that has met the reserve price
    FinalizeAuction {
        token_id: TokenId,
    },
    /// The bidder can void an expired Auction that has not been determined
    /// by the seller
    VoidAuction {
        token_id: TokenId,
    },
}

/// Options when querying for Asks and Bids
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub filter_expiry: Option<Timestamp>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenTimestampOffset {
    pub token_id: TokenId,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenPriceOffset {
    pub token_id: TokenId,
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get the config for the contract
    /// Return type: `ConfigResponse`
    Config {},
    /// Get the auction for a specific NFT
    /// Return type: `AuctionResponse`
    Auction {
        token_id: TokenId,
    },
    /// Get the auctions sorted by the start time
    /// Return type: `AuctionsResponse`
    AuctionsByStartTime {
        query_options: QueryOptions<TokenTimestampOffset>
    },
    /// Get the auctions sorted by the end time
    /// Return type: `AuctionsResponse`
    AuctionsByEndTime {
        query_options: QueryOptions<TokenTimestampOffset>
    },
    /// Get the auctions sorted by the highest bid price
    /// Return type: `AuctionsResponse`
    AuctionsByHighestBidPrice {
        query_options: QueryOptions<TokenPriceOffset>
    },
    /// Get all auctions sorted by seller and end time
    /// Return type: `AuctionsResponse`
    AuctionsBySellerEndTime {
        seller: String,
        query_options: QueryOptions<TokenTimestampOffset>
    },
    /// Get all auctions sorted by bidder and end time
    /// Return type: `AuctionsResponse`
    AuctionsByBidderEndTime {
        bidder: String,
        query_options: QueryOptions<TokenTimestampOffset>
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AuctionResponse {
    pub auction: Option<Auction>,
    pub auction_status: Option<AuctionStatus>,
    pub is_reserve_price_met: Option<bool>,
    pub next_bid_min: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AuctionsResponse {
    pub auctions: Vec<Auction>,
}
