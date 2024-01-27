use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub token_symbol: String,
    pub token_contract_address: Addr,
}


#[cw_serde]
pub enum ExecuteMsg {
    // Receive Filter
    Receive(Cw20ReceiveMsg),
}


#[cw_serde]
pub enum ReceiveMsg {
    AnExecuteMsg {},
}

///~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////// Query
///~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    GetAdmin {},
}


#[cw_serde]
pub struct AdminResponse {
    pub admin: String,
}
