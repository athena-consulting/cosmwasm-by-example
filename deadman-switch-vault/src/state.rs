use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub beneficiary: Addr,
    pub denom: String,
    pub heartbeat_window: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const LAST_HEARTBEAT: Item<u64> = Item::new("last_heartbeat");
