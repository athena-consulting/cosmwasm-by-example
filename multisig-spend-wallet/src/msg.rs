use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    pub owners: Vec<String>,
    pub threshold: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    ProposeSpend { recipient: String, amount: Coin },
    Approve { proposal_id: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ProposalResponse)]
    Proposal { proposal_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owners: Vec<String>,
    pub threshold: u64,
    pub next_proposal_id: u64,
}

#[cw_serde]
pub struct ProposalResponse {
    pub id: u64,
    pub proposer: String,
    pub recipient: String,
    pub amount: Coin,
    pub approvals: Vec<String>,
    pub executed: bool,
}
