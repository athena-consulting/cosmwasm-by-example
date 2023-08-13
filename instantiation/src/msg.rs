use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
/*
variables to be passed to contract at instantiation.
secret variables or any variable that is open to a replay attack should not be 
part of the InstantiateMsg 
*/
pub struct InstantiateMsg {
    pub sent_message: String
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetMessageResponse)]
    GetMessage {}
}

#[cw_serde]
pub struct GetMessageResponse {
    pub message: String
}