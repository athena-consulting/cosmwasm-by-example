use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    /* Count is a state variable which means that the data will be stored on the blockchain */
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
