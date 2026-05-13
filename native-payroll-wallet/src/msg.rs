use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub period_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Fund {},
    SetEmployee {
        employee: String,
        salary: Uint128,
        last_claim: Option<u64>,
    },
    RemoveEmployee {
        employee: String,
    },
    Claim {},
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Employee { employee: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub period_seconds: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct EmployeeResponse {
    pub employee: String,
    pub configured: bool,
    pub salary: Uint128,
    pub last_claim: u64,
    pub next_claim: u64,
}
