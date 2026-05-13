use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub denom: String,
}

#[cw_serde]
pub enum GiftStatus {
    Open,
    Claimed,
    Canceled,
}

#[cw_serde]
pub struct Gift {
    pub creator: Addr,
    pub recipient: Addr,
    pub amount: Uint128,
    pub status: GiftStatus,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_GIFT_ID: Item<u64> = Item::new("next_gift_id");
pub const GIFTS: Map<u64, Gift> = Map::new("gifts");
