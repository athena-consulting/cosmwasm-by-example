use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct RecipientShare {
    pub address: Addr,
    pub share: Uint128,
}

#[cw_serde]
pub struct Config {
    pub recipients: Vec<RecipientShare>,
    pub total_shares: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
