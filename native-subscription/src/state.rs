use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub denom: String,
    pub price: Uint128,
    pub period_seconds: u64,
}

#[cw_serde]
pub struct Subscription {
    pub expires_at: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SUBSCRIPTIONS: Map<&Addr, Subscription> = Map::new("subscriptions");
