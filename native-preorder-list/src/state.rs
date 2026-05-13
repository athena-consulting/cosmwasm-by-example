use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub item: String,
    pub unit_price: Uint128,
    pub max_units: Option<u64>,
    pub sold_units: u64,
    pub balance: Uint128,
    pub next_order_id: u64,
}

#[cw_serde]
pub struct Order {
    pub buyer: Addr,
    pub recipient: Addr,
    pub quantity: u64,
    pub paid: Uint128,
    pub note: Option<String>,
    pub fulfilled: bool,
    pub refunded: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ORDERS: Map<u64, Order> = Map::new("orders");
