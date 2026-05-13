use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub beneficiary: String,
    pub denom: String,
    pub target: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Fund {},
    Release {},
    UpdateBeneficiary { beneficiary: String },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub beneficiary: String,
    pub denom: String,
    pub target: Uint128,
    pub saved: Uint128,
    pub released: bool,
}
