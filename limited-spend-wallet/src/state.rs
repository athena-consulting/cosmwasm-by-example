use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub spender: Addr,
    pub denom: String,
    pub limit: Uint128,
    pub window_size: u64,
    pub window_start: u64,
    pub spent: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
