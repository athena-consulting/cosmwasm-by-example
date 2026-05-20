use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Poll {
    pub creator: Addr,
    pub question: String,
    pub closes_at_height: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
}

pub const POLL: Item<Poll> = Item::new("poll");
pub const VOTES: Map<&Addr, bool> = Map::new("votes");
