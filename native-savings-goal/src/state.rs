use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub beneficiary: Addr,
    pub denom: String,
    pub target: Uint128,
    pub saved: Uint128,
    pub released: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
