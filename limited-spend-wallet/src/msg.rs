use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub spender: String,
    pub denom: String,
    pub limit: Uint128,
    pub window_size: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    Spend { recipient: String, amount: Uint128 },
    UpdateSpender { spender: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(WindowResponse)]
    Window {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub spender: String,
    pub denom: String,
    pub limit: Uint128,
    pub window_size: u64,
}

#[cw_serde]
pub struct WindowResponse {
    pub window_start: u64,
    pub spent: Uint128,
    pub remaining: Uint128,
}
