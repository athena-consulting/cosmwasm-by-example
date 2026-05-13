use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL_ALLOCATED: Item<Uint128> = Item::new("total_allocated");
pub const REWARDS: Map<&Addr, Uint128> = Map::new("rewards");
