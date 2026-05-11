use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub claim_amount: Uint128,
    pub cooldown_seconds: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LAST_CLAIM: Map<&Addr, u64> = Map::new("last_claim");
