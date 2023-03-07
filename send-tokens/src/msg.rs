use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr, Coin};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SendTokens {amount: Uint128, denom: String, to: Addr}
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

}

#[cw_serde]
pub struct BalanceResponse {
    pub amount: Coin,
}
