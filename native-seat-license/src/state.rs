use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub seat_price: Uint128,
    pub max_seats: u64,
    pub sold_seats: u64,
    pub balance: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SEATS: Map<&Addr, bool> = Map::new("seats");
