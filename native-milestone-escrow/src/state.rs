use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub default_arbiter: Option<Addr>,
}

#[cw_serde]
pub enum EscrowStatus {
    Active,
    Released,
    Refunded,
}

#[cw_serde]
pub struct Escrow {
    pub id: u64,
    pub buyer: Addr,
    pub seller: Addr,
    pub arbiter: Addr,
    pub amount: Uint128,
    pub description: String,
    pub timeout: Option<u64>,
    pub created_at: u64,
    pub settled_at: Option<u64>,
    pub status: EscrowStatus,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const ESCROWS: Map<u64, Escrow> = Map::new("escrows");
