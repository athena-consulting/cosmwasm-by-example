use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

#[cw_serde]
pub struct Profile {
    pub owner: Addr,
    pub display_name: String,
    pub bio: Option<String>,
    pub website: Option<String>,
}

pub const PROFILES: Map<&Addr, Profile> = Map::new("profiles");
