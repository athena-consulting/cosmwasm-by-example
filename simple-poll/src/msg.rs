use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_question_len: Option<u16>,
    pub max_option_len: Option<u16>,
    pub max_options: Option<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePoll {
        question: String,
        options: Vec<String>,
        closes_at: Option<u64>,
    },
    Vote {
        poll_id: u64,
        option_index: u8,
    },
    ClosePoll {
        poll_id: u64,
    },
    UpdateConfig {
        max_question_len: Option<u16>,
        max_option_len: Option<u16>,
        max_options: Option<u8>,
        new_owner: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(PollResponse)]
    Poll { poll_id: u64 },
    #[returns(PollsResponse)]
    Polls {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(VoteResponse)]
    Vote { poll_id: u64, voter: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub max_question_len: u16,
    pub max_option_len: u16,
    pub max_options: u8,
}

#[cw_serde]
pub struct PollOptionResponse {
    pub index: u8,
    pub label: String,
    pub votes: u64,
}

#[cw_serde]
pub struct PollResponse {
    pub id: u64,
    pub creator: String,
    pub question: String,
    pub options: Vec<PollOptionResponse>,
    pub closes_at: Option<u64>,
    pub closed: bool,
    pub total_votes: u64,
}

#[cw_serde]
pub struct PollsResponse {
    pub polls: Vec<PollResponse>,
}

#[cw_serde]
pub struct VoteResponse {
    pub poll_id: u64,
    pub voter: String,
    pub option_index: Option<u8>,
}
