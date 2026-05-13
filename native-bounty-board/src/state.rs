use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
}

#[cw_serde]
pub struct Bounty {
    pub sponsor: Addr,
    pub title: String,
    pub amount: Uint128,
    pub settled: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const BOUNTIES: Map<u64, Bounty> = Map::new("bounties");
