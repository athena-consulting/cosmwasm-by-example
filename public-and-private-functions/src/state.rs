use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    /* Addr is a cosmwasm-std primitive that helps validate addresses on the Cosmos ecosystem,
    a wrapper for a string that also validates the address in use. */
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
