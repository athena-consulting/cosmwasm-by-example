use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

#[cw_serde]
pub struct Contact {
    pub address: Addr,
    pub note: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CONTACTS: Map<String, Contact> = Map::new("contacts");
