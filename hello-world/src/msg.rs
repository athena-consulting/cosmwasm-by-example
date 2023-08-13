use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HelloWorldResponse)]
    HelloWorld {}
}

#[cw_serde]
pub struct HelloWorldResponse {
    pub hello_world_message: String
}
