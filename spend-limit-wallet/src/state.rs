use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub default_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Allowance {
    pub limit: Uint128,
    pub spent: Uint128,
    pub period_start: u64,
    pub period: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const ALLOWANCES: Map<&Addr, Allowance> = Map::new("allowances");
