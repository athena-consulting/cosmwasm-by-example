use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<String>,
    pub threshold: u64,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    ProposeSpend {
        recipient: String,
        amount: Uint128,
        description: String,
    },
    Approve {
        proposal_id: u64,
    },
    Execute {
        proposal_id: u64,
    },
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
    pub members: Vec<String>,
    pub threshold: u64,
    pub denom: String,
    pub next_proposal_id: u64,
}

#[cw_serde]
pub struct ProposalResponse {
    pub id: u64,
    pub proposer: String,
    pub recipient: String,
    pub amount: Uint128,
    pub description: String,
    pub approvals: Vec<String>,
    pub executed: bool,
}
