use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub next_bill_id: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub enum BillStatus {
    Open,
    Settled,
    Canceled,
}

#[cw_serde]
pub struct Bill {
    pub id: u64,
    pub creator: Addr,
    pub recipient: Addr,
    pub participants: Vec<Addr>,
    pub share_amount: Uint128,
    pub total_amount: Uint128,
    pub paid_count: u64,
    pub status: BillStatus,
    pub memo: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BILLS: Map<u64, Bill> = Map::new("bills");
pub const PAID: Map<(u64, &Addr), bool> = Map::new("paid");
