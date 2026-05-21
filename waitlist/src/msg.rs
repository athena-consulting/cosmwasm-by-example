use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::EntryStatus;

#[cw_serde]
pub struct InstantiateMsg {
    pub max_note_length: Option<u32>,
    pub max_entries: Option<u32>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Join {
        note: Option<String>,
    },
    UpdateNote {
        note: Option<String>,
    },
    Leave {},
    Admit {
        member: String,
    },
    Remove {
        member: String,
    },
    UpdateConfig {
        owner: Option<String>,
        max_note_length: Option<u32>,
        max_entries: Option<u32>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(EntryResponse)]
    Entry { member: String },
    #[returns(StatsResponse)]
    Stats {},
    #[returns(ListEntriesResponse)]
    ListEntries {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub max_note_length: u32,
    pub max_entries: u32,
}

#[cw_serde]
pub struct EntryView {
    pub member: String,
    pub note: Option<String>,
    pub position: u64,
    pub joined_at_height: u64,
    pub status: EntryStatus,
}

#[cw_serde]
pub struct EntryResponse {
    pub entry: Option<EntryView>,
}

#[cw_serde]
pub struct StatsResponse {
    pub waiting: u32,
    pub admitted: u32,
    pub active: u32,
    pub next_position: u64,
}

#[cw_serde]
pub struct ListEntriesResponse {
    pub entries: Vec<EntryView>,
}

#[cw_serde]
pub struct MigrateMsg {}
