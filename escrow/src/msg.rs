use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub arbiter: String,
    pub recipient: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Approve {},
    Refund {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub arbiter: String,
    pub recipient: String,
    pub source: String,
}
