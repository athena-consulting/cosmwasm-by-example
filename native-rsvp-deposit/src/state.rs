use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub forfeiture_recipient: Addr,
    pub denom: String,
    pub deposit_amount: Uint128,
    pub event_time: u64,
}

#[cw_serde]
pub struct Attendee {
    pub checked_in: bool,
    pub settled: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ATTENDEES: Map<&Addr, Attendee> = Map::new("attendees");
