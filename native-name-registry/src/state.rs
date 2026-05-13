use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub registration_fee: Uint128,
    pub registration_seconds: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct NameRecord {
    pub owner: Addr,
    pub value: String,
    pub expires_at: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NAMES: Map<&str, NameRecord> = Map::new("names");
