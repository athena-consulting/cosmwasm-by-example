use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub beneficiary: String,
    pub denom: String,
    pub heartbeat_window: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    Heartbeat {},
    UpdateConfig {
        owner: Option<String>,
        beneficiary: Option<String>,
        heartbeat_window: Option<u64>,
    },
    Claim {
        recipient: Option<String>,
        amount: Option<Uint128>,
    },
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
    pub beneficiary: String,
    pub denom: String,
    pub heartbeat_window: u64,
}

#[cw_serde]
pub struct StatusResponse {
    pub owner: String,
    pub beneficiary: String,
    pub denom: String,
    pub heartbeat_window: u64,
    pub last_heartbeat: u64,
    pub expires_at: u64,
    pub expired: bool,
}
