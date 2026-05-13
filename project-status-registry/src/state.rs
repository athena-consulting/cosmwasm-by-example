use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::msg::ProjectStatus;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

#[cw_serde]
pub struct Project {
    pub id: String,
    pub title: String,
    pub status: ProjectStatus,
    pub note: String,
    pub updated_by: Addr,
    pub updated_at: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PROJECTS: Map<&str, Project> = Map::new("projects");
