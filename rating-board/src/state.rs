use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub max_rating: u8,
    pub next_item_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct RatedItem {
    pub creator: Addr,
    pub name: String,
    pub description: Option<String>,
    pub total_rating: u64,
    pub rating_count: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ITEMS: Map<u64, RatedItem> = Map::new("items");
pub const RATINGS: Map<(u64, &Addr), u8> = Map::new("ratings");
