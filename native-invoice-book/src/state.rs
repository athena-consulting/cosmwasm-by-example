use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub balance: Uint128,
    pub next_invoice_id: u64,
}

#[cw_serde]
pub struct Invoice {
    pub payer: Option<Addr>,
    pub amount: Uint128,
    pub memo: Option<String>,
    pub paid: bool,
    pub paid_by: Option<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const INVOICES: Map<u64, Invoice> = Map::new("invoices");
