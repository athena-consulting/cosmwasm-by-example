use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, Uint64};
use cw_storage_plus::{Item, Map};
use cw_utils::{Duration, Scheduled};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Timelock {
    pub admins: Vec<Addr>,
    pub proposers: Vec<Addr>,
    pub min_time_delay: Duration,
    pub frozen: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Operation {
    pub id: Uint64,
    pub status: OperationStatus,
    pub proposer: Addr,
    pub executors: Option<Vec<Addr>>,
    pub execution_time: Scheduled,
    pub target: Addr,
    pub data: Binary,
    pub title: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OperationStatus {
    Pending,
    Ready,
    Done,
}

pub const CONFIG: Item<Timelock> = Item::new("timelock");
pub const OPERATION_LIST: Map<u64, Operation> = Map::new("operation_list");
pub const OPERATION_SEQ: Item<Uint64> = Item::new("operation_seq");
