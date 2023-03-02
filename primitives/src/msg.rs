use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
}

#[cw_serde]
pub enum ExecuteMsg {

}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetOwnerResponse)]
    GetOwner {},

    #[returns(GetIntegerResponse)]
    Integer {}
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetOwnerResponse {
    pub owner: Addr,
}

#[cw_serde]
pub struct GetIntegerResponse {
    pub works: bool
}