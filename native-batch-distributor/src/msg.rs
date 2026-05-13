use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    Distribute { payments: Vec<Payment> },
    UpdateAdmin { admin: String },
}

#[cw_serde]
pub struct Payment {
    pub recipient: String,
    pub amount: Uint128,
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Balance {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub denom: String,
}

#[cw_serde]
pub struct ContractBalanceResponse {
    pub denom: String,
    pub amount: Uint128,
}
