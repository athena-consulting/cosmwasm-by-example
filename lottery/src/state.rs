use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub ticket_price: u128,
    pub ticket_denom: String,
    pub winner: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TICKETS: Item<Vec<String>> = Item::new("tickets");
