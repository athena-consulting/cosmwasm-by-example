use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CosmosMsg, StdError, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub denom: String,
    pub goal: Uint128,
    pub start: Timestamp,
    pub deadline: Timestamp,
    pub name: String,
    pub description: String,
}

impl Config {
    pub fn validate(&self) -> Result<(), StdError> {
        if self.goal <= Uint128::zero() {
            return Err(StdError::generic_err(
                "goal must be greater than 0".to_string(),
            ));
        }
        if self.start >= self.deadline {
            return Err(StdError::generic_err(
                "start must be before deadline".to_string(),
            ));
        }
        // description must be less than 256 characters
        if self.description.len() > 256 {
            return Err(StdError::generic_err(
                "description must be less than 256 characters".to_string(),
            ));
        }
        // title must be less than 32 characters
        if self.name.len() > 32 {
            return Err(StdError::generic_err(
                "title must be less than 32 characters".to_string(),
            ));
        }
        Ok(())
    }
}


pub const CONFIG: Item<Config> = Item::new("config");
pub const SHARES: Map<Addr,Uint128> = Map::new("shares");
pub const TOTAL_SHARES: Item<Uint128> = Item::new("total_shares");
pub const EXECUTE_MSG: Item<Option<CosmosMsg>> = Item::new("execute_msg");


