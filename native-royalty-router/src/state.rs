use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub shares: Vec<RoyaltyShare>,
}

#[cw_serde]
pub struct RoyaltyShare {
    pub recipient: Addr,
    pub bps: u16,
}

pub const CONFIG: Item<Config> = Item::new("config");
