use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub owner: Option<String>,
    pub spender: String,
    pub period_seconds: u64,
    pub period_limit: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    Withdraw {
        amount: Uint128,
        recipient: Option<String>,
    },
    UpdateConfig {
        owner: Option<String>,
        spender: Option<String>,
        period_seconds: Option<u64>,
        period_limit: Option<Uint128>,
        reset_period: bool,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(AllowanceResponse)]
    Allowance {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
    pub spender: Addr,
    pub denom: String,
    pub period_seconds: u64,
    pub period_limit: Uint128,
}

#[cw_serde]
pub struct AllowanceResponse {
    pub owner: Addr,
    pub spender: Addr,
    pub denom: String,
    pub current_period_start: u64,
    pub current_period_end: u64,
    pub period_limit: Uint128,
    pub spent_this_period: Uint128,
    pub remaining_this_period: Uint128,
    pub contract_balance: Uint128,
}
