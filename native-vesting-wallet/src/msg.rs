use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub recipient: String,
    pub denom: String,
    pub start_time: u64,
    pub cliff_time: Option<u64>,
    pub end_time: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {},
    Deposit {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StatusResponse)]
    Status {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub recipient: String,
    pub denom: String,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub total_amount: Uint128,
    pub released_amount: Uint128,
}

#[cw_serde]
pub struct StatusResponse {
    pub now: u64,
    pub total_amount: Uint128,
    pub vested_amount: Uint128,
    pub released_amount: Uint128,
    pub claimable_amount: Uint128,
    pub locked_amount: Uint128,
}
