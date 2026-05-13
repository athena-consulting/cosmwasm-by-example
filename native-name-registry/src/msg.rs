use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub registration_fee: Uint128,
    pub registration_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Register { name: String, value: String },
    Renew { name: String },
    UpdateValue { name: String, value: String },
    Release { name: String },
    Withdraw { recipient: String, amount: Uint128 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Name { name: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub registration_fee: Uint128,
    pub registration_seconds: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct NameResponse {
    pub name: String,
    pub owner: Option<String>,
    pub value: Option<String>,
    pub expires_at: Option<u64>,
    pub is_expired: bool,
}
