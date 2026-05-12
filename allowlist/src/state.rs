use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const ALLOWLIST: Map<&Addr, String> = Map::new("allowlist");
