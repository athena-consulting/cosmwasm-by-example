use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub next_budget_id: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct Budget {
    pub id: u64,
    pub manager: Addr,
    pub label: String,
    pub cap: Uint128,
    pub spent: Uint128,
    pub active: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BUDGETS: Map<u64, Budget> = Map::new("budgets");
