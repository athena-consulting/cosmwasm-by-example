use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Lock { unlock_time: u64 },
    Extend { id: u64, unlock_time: u64 },
    Claim { id: u64 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Lock { id: u64 },
    NextId {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub denom: String,
}

#[cw_serde]
pub struct LockResponse {
    pub id: u64,
    pub owner: String,
    pub amount: Uint128,
    pub unlock_time: u64,
    pub claimed: bool,
}

#[cw_serde]
pub struct NextIdResponse {
    pub next_id: u64,
}
