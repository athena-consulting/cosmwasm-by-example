use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

#[cw_serde]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub body: String,
    pub expires_at: u64,
    pub archived: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ANNOUNCEMENTS: Map<&str, Announcement> = Map::new("announcements");
