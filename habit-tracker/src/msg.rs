use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub min_gap: Option<u64>,
    pub max_gap: Option<u64>,
    pub max_note_len: Option<u16>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CheckIn {
        note: Option<String>,
    },
    Reset {},
    UpdateConfig {
        min_gap: Option<u64>,
        max_gap: Option<u64>,
        max_note_len: Option<u16>,
        new_owner: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(RecordResponse)]
    Record { user: String },
    #[returns(RecordsResponse)]
    Records {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub min_gap: u64,
    pub max_gap: u64,
    pub max_note_len: u16,
}

#[cw_serde]
pub struct RecordResponse {
    pub user: String,
    pub last_check_in_height: u64,
    pub current_streak: u64,
    pub best_streak: u64,
    pub total_check_ins: u64,
    pub note: Option<String>,
}

#[cw_serde]
pub struct RecordsResponse {
    pub records: Vec<RecordResponse>,
}
