use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub max_items_per_user: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InventoryItem {
    pub item_id: u64,
    pub owner: Addr,
    pub sku: String,
    pub name: String,
    pub quantity: Uint128,
    pub note: Option<String>,
    pub created_at_height: u64,
    pub updated_at_height: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const USER_NEXT_ID: Map<&Addr, u64> = Map::new("user_next_id");
pub const USER_ITEM_COUNT: Map<&Addr, u32> = Map::new("user_item_count");
pub const USER_ITEMS: Map<(&Addr, u64), InventoryItem> = Map::new("user_items");
