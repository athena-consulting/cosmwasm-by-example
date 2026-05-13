use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub price: Uint128,
    pub period_seconds: u64,
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Subscribe {},
    Withdraw {
        recipient: Option<String>,
        amount: Option<Uint128>,
    },
    UpdateConfig {
        price: Option<Uint128>,
        period_seconds: Option<u64>,
        admin: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(SubscriptionResponse)]
    Subscription { subscriber: String },

    #[returns(IsActiveResponse)]
    IsActive { subscriber: String },

    #[returns(TreasuryResponse)]
    Treasury {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: Addr,
    pub denom: String,
    pub price: Uint128,
    pub period_seconds: u64,
}

#[cw_serde]
pub struct SubscriptionResponse {
    pub subscriber: Addr,
    pub expires_at: u64,
    pub active: bool,
    pub remaining_seconds: u64,
}

#[cw_serde]
pub struct IsActiveResponse {
    pub active: bool,
}

#[cw_serde]
pub struct TreasuryResponse {
    pub denom: String,
    pub balance: Uint128,
}
