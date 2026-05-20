use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub max_open_tasks: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UserStats {
    pub next_id: u64,
    pub open_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Task {
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at_height: u64,
    pub completed_at_height: Option<u64>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const USER_STATS: Map<&Addr, UserStats> = Map::new("user_stats");
pub const TASKS: Map<(&Addr, u64), Task> = Map::new("tasks");
