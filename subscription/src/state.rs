use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub provider: String,
    pub price: u128,
    pub denom: String,
    pub interval_seconds: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SUBSCRIPTIONS: Map<&str, u64> = Map::new("subscriptions");
