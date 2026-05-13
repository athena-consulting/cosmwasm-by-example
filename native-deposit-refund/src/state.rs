use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub denom: String,
    pub deposit_amount: Uint128,
    pub refund_deadline: u64,
}

#[cw_serde]
pub struct Deposit {
    pub amount: Uint128,
    pub deposited_at: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DEPOSITS: Map<&Addr, Deposit> = Map::new("deposits");
