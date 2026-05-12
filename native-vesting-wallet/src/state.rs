use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub recipient: Addr,
    pub denom: String,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub total_amount: Uint128,
    pub released_amount: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
