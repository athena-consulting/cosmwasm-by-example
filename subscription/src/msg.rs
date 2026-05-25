use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub provider: String,
    pub price: String,
    pub denom: String,
    pub interval_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Subscribe {},
    Claim {},
    Cancel {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(SubResponse)]
    Subscription { address: String },
}

#[cw_serde]
pub struct SubResponse {
    pub next_claim: u64,
}
