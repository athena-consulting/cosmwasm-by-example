use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub title: String,
    pub max_message_len: u32,
    pub entry_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Entry {
    pub display_name: String,
    pub message: String,
    pub signed_at_height: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ENTRIES: Map<&Addr, Entry> = Map::new("entries");
