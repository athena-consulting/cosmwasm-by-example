use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub spender: Addr,
    pub denom: String,
    pub period_seconds: u64,
    pub period_limit: Uint128,
    pub current_period_start: u64,
    pub spent_this_period: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
