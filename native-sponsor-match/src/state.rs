use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub beneficiary: Addr,
    pub denom: String,
    pub match_bps: u16,
    pub match_pool: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
