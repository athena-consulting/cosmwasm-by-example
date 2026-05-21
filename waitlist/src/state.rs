use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub max_note_length: u32,
    pub max_entries: u32,
}

#[cw_serde]
pub struct Entry {
    pub member: Addr,
    pub note: Option<String>,
    pub position: u64,
    pub joined_at_height: u64,
    pub status: EntryStatus,
}

#[cw_serde]
pub enum EntryStatus {
    Waiting,
    Admitted,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_POSITION: Item<u64> = Item::new("next_position");
pub const WAITING_COUNT: Item<u32> = Item::new("waiting_count");
pub const ADMITTED_COUNT: Item<u32> = Item::new("admitted_count");
pub const ENTRIES: Map<&Addr, Entry> = Map::new("entries");
