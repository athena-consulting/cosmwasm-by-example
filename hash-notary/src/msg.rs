use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Register {
        digest: String,
        label: String,
        memo: Option<String>,
    },
    UpdateMemo {
        id: u64,
        memo: Option<String>,
    },
    Revoke {
        id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StateResponse)]
    GetState {},
    #[returns(RecordResponse)]
    GetRecord { id: u64 },
    #[returns(RecordResponse)]
    GetRecordByDigest { digest: String },
    #[returns(ListRecordsResponse)]
    ListRecords {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct StateResponse {
    pub owner: Addr,
    pub total_records: u64,
}

#[cw_serde]
pub struct RecordResponse {
    pub id: u64,
    pub digest: String,
    pub owner: Addr,
    pub created_at: u64,
    pub label: String,
    pub memo: Option<String>,
    pub active: bool,
}

#[cw_serde]
pub struct ListRecordsResponse {
    pub records: Vec<RecordResponse>,
}
