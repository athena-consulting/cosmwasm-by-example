use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub default_period: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetAllowance {
        spender: String,
        limit: Uint128,
        period: Option<u64>,
    },
    RemoveAllowance {
        spender: String,
    },
    Spend {
        recipient: String,
        amount: Uint128,
    },
    UpdateConfig {
        owner: Option<String>,
        default_period: Option<u64>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AllowanceResponse)]
    Allowance { spender: String },
    #[returns(SpendableResponse)]
    Spendable { spender: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub default_period: u64,
}

#[cw_serde]
pub struct AllowanceResponse {
    pub spender: String,
    pub limit: Uint128,
    pub spent: Uint128,
    pub remaining: Uint128,
    pub period_start: u64,
    pub period: u64,
}

#[cw_serde]
pub struct SpendableResponse {
    pub spender: String,
    pub amount: Uint128,
    pub period_start: u64,
    pub period: u64,
}
