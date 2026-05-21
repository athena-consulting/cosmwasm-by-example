use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub min_gap: u64,
    pub max_gap: u64,
    pub max_note_len: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct HabitRecord {
    pub last_check_in_height: u64,
    pub current_streak: u64,
    pub best_streak: u64,
    pub total_check_ins: u64,
    pub note: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const RECORDS: Map<&Addr, HabitRecord> = Map::new("records");
