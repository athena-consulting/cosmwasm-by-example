use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub question: String,
    pub closes_at_height: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Vote { approve: bool },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(PollResponse)]
    Poll {},
    #[returns(VoteResponse)]
    Vote { voter: String },
}

#[cw_serde]
pub struct PollResponse {
    pub creator: String,
    pub question: String,
    pub closes_at_height: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub open: bool,
    pub result: String,
}

#[cw_serde]
pub struct VoteResponse {
    pub voter: String,
    pub voted: bool,
    pub approve: Option<bool>,
}
