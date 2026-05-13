use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub period_seconds: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct Employee {
    pub salary: Uint128,
    pub last_claim: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const EMPLOYEES: Map<&Addr, Employee> = Map::new("employees");
