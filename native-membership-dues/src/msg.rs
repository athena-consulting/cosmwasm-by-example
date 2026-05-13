use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub fee: Uint128,
    pub period_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    PayDues { member: Option<String> },
    Withdraw { recipient: String, amount: Uint128 },
    UpdateTerms { fee: Uint128, period_seconds: u64 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Member { member: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub fee: Uint128,
    pub period_seconds: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct MemberResponse {
    pub member: String,
    pub expires_at: u64,
    pub active: bool,
}
