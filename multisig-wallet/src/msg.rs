use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::CosmosMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub signers: Vec<String>,
    pub threshold: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Propose { msg: CosmosMsg },
    Approve { proposal_id: u64 },
    Execute { proposal_id: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ProposalResponse)]
    Proposal { id: u64 },
}

#[cw_serde]
pub struct ProposalResponse {
    pub approvals: u64,
}
