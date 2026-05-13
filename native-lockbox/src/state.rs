use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub denom: String,
}

#[cw_serde]
pub struct Lock {
    pub owner: Addr,
    pub amount: Uint128,
    pub unlock_time: u64,
    pub claimed: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const LOCKS: Map<u64, Lock> = Map::new("locks");
