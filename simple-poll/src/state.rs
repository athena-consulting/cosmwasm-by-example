use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub max_question_len: u16,
    pub max_option_len: u16,
    pub max_options: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PollOption {
    pub label: String,
    pub votes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Poll {
    pub id: u64,
    pub creator: Addr,
    pub question: String,
    pub options: Vec<PollOption>,
    pub closes_at: Option<u64>,
    pub closed: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_POLL_ID: Item<u64> = Item::new("next_poll_id");
pub const POLLS: Map<u64, Poll> = Map::new("polls");
pub const VOTES: Map<(u64, &Addr), u8> = Map::new("votes");
