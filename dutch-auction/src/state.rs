use cosmwasm_std::{Addr, StdResult, Storage, Timestamp, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub seller: Addr,
    pub denom: String,
    pub starting_price: Uint128,
    pub start_at: Timestamp,
    pub expires_at: Timestamp,
    pub discount_rate: Uint128,
    pub nft_address: Addr,
    pub nft_id: Uint128,
}

impl State {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        STATE.save(storage, self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<State> {
        STATE.load(storage)
    }

    pub fn remove(&self, storage: &mut dyn Storage) -> StdResult<()> {
        Ok(STATE.remove(storage))
    }
}

pub const STATE: Item<State> = Item::new("state");
