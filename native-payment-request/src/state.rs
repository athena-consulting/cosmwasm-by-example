use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub merchant: Addr,
    pub default_recipient: Addr,
    pub denom: String,
}

#[cw_serde]
pub struct Invoice {
    pub id: u64,
    pub payer: Option<Addr>,
    pub recipient: Addr,
    pub amount: Uint128,
    pub description: String,
    pub due_at: Option<u64>,
    pub paid: bool,
    pub canceled: bool,
    pub paid_at: Option<u64>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const INVOICES: Map<u64, Invoice> = Map::new("invoices");
