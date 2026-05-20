use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Vesting {
    pub creator: Addr,
    pub beneficiary: Addr,
    pub total: Coin,
    pub claimed: Uint128,
    pub start_height: u64,
    pub cliff_height: u64,
    pub end_height: u64,
}

pub const VESTING: Item<Vesting> = Item::new("vesting");
