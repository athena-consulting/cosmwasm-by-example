use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub recipients: Vec<RecipientShareMsg>,
}

#[cw_serde]
pub struct RecipientShareMsg {
    pub address: String,
    pub share: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Distribute {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub recipients: Vec<RecipientShareMsg>,
    pub total_shares: Uint128,
}
