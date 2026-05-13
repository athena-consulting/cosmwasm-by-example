use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    SetReward { recipient: String, amount: Uint128 },
    Claim {},
    UpdateOwner { owner: String },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Reward { recipient: String },
    Pool {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub total_allocated: Uint128,
}

#[cw_serde]
pub struct RewardResponse {
    pub recipient: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct PoolResponse {
    pub denom: String,
    pub balance: Uint128,
    pub total_allocated: Uint128,
}
