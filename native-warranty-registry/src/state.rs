use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub registration_fee: Uint128,
    pub coverage_seconds: u64,
    pub next_claim_id: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct Product {
    pub serial: String,
    pub owner: Addr,
    pub metadata: String,
    pub expires_at: u64,
}

#[cw_serde]
pub enum ClaimStatus {
    Open,
    Approved,
    Rejected,
}

#[cw_serde]
pub struct WarrantyClaim {
    pub id: u64,
    pub serial: String,
    pub claimant: Addr,
    pub note: String,
    pub status: ClaimStatus,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PRODUCTS: Map<&str, Product> = Map::new("products");
pub const CLAIMS: Map<u64, WarrantyClaim> = Map::new("claims");
