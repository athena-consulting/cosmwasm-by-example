use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    PublicFunction { param: String },
}

#[cw_serde]
pub enum QueryMsg {}
