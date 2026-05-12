use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Member {
    pub address: Addr,
    pub share: u64,
}

#[cw_serde]
pub struct Config {
    pub denom: String,
    pub members: Vec<Member>,
    pub total_shares: u128,
    pub remainder_recipient: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
