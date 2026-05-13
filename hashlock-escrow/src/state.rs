use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Coin, Timestamp};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Escrow {
    pub sender: Addr,
    pub recipient: Addr,
    pub hash: Binary,
    pub deadline: Timestamp,
    pub amount: Coin,
    pub claimed: bool,
    pub refunded: bool,
}

pub const ESCROW: Item<Escrow> = Item::new("escrow");
