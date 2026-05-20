use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub title: String,
    pub max_message_len: Option<u32>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Sign {
        display_name: String,
        message: String,
    },
    RemoveEntry {
        signer: String,
    },
    UpdateConfig {
        title: Option<String>,
        max_message_len: Option<u32>,
        new_owner: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(EntryResponse)]
    Entry { signer: String },
    #[returns(EntriesResponse)]
    Entries {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub title: String,
    pub max_message_len: u32,
    pub entry_count: u64,
}

#[cw_serde]
pub struct EntryResponse {
    pub signer: String,
    pub display_name: String,
    pub message: String,
    pub signed_at_height: u64,
}

#[cw_serde]
pub struct EntriesResponse {
    pub entries: Vec<EntryResponse>,
}
