use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owners: Vec<Addr>,
    pub threshold: u64,
    pub next_proposal_id: u64,
}

#[cw_serde]
pub struct Proposal {
    pub id: u64,
    pub proposer: Addr,
    pub recipient: Addr,
    pub amount: Coin,
    pub approvals: Vec<Addr>,
    pub executed: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
