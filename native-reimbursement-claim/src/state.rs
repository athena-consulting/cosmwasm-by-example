use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub balance: Uint128,
    pub next_claim_id: u64,
}

#[cw_serde]
pub struct ReimbursementClaim {
    pub claimant: Addr,
    pub amount: Uint128,
    pub description: Option<String>,
    pub paid: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CLAIMS: Map<u64, ReimbursementClaim> = Map::new("claims");
