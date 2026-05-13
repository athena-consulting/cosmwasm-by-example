use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub organizer: Addr,
    pub denom: String,
    pub price: Uint128,
    pub max_tickets: u64,
    pub sold: u64,
    pub revenue: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PURCHASES: Map<&Addr, u64> = Map::new("purchases");
