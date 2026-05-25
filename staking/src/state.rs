use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub token_denom: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STAKES: Map<&str, u128> = Map::new("stakes");
