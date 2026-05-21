use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::msg::RecordResponse;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub total_records: u64,
}

#[cw_serde]
pub struct Record {
    pub id: u64,
    pub digest: String,
    pub owner: Addr,
    pub created_at: u64,
    pub label: String,
    pub memo: Option<String>,
    pub active: bool,
}

impl From<Record> for RecordResponse {
    fn from(record: Record) -> Self {
        RecordResponse {
            id: record.id,
            digest: record.digest,
            owner: record.owner,
            created_at: record.created_at,
            label: record.label,
            memo: record.memo,
            active: record.active,
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const RECORD_SEQ: Item<u64> = Item::new("record_seq");
pub const RECORDS: Map<u64, Record> = Map::new("records");
pub const DIGEST_INDEX: Map<&str, u64> = Map::new("digest_index");
