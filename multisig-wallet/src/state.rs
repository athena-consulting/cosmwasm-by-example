use cosmwasm_schema::cw_serde;
use cosmwasm_std::CosmosMsg;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub signers: Vec<String>,
    pub threshold: u64,
}

#[cw_serde]
pub struct Proposal {
    pub msg: CosmosMsg,
    pub approvals: Vec<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");
