use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub slot_price: Uint128,
    pub rental_seconds: u64,
    pub next_slot_id: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct Slot {
    pub id: u64,
    pub label: String,
    pub active: bool,
    pub renter: Option<Addr>,
    pub message: Option<String>,
    pub paid_until: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SLOTS: Map<u64, Slot> = Map::new("slots");
