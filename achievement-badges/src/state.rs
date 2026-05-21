use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub name: String,
    pub description: Option<String>,
    pub max_badges_per_holder: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Badge {
    pub badge_id: String,
    pub title: String,
    pub description: String,
    pub creator: Addr,
    pub archived: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Award {
    pub badge_id: String,
    pub holder: Addr,
    pub awarded_by: Addr,
    pub awarded_at_height: u64,
    pub note: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BADGES: Map<&str, Badge> = Map::new("badges");
pub const ISSUERS: Map<&Addr, bool> = Map::new("issuers");
pub const AWARDS: Map<(&Addr, &str), Award> = Map::new("awards");
pub const HOLDER_COUNTS: Map<&Addr, u32> = Map::new("holder_counts");
