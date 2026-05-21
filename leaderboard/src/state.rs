use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub max_name_length: u32,
    pub max_metadata_length: u32,
}

#[cw_serde]
pub struct ScoreEntry {
    pub player: Addr,
    pub display_name: String,
    pub score: u64,
    pub metadata: Option<String>,
    pub updated_at: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PLAYER_COUNT: Item<u64> = Item::new("player_count");
pub const SCORES: Map<&Addr, ScoreEntry> = Map::new("scores");
