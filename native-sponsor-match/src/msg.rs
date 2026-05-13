use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub beneficiary: String,
    pub denom: String,
    pub match_bps: u16,
}

#[cw_serde]
pub enum ExecuteMsg {
    FundMatchPool {},
    Donate {},
    UpdateConfig {
        beneficiary: Option<String>,
        match_bps: Option<u16>,
    },
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
    pub match_bps: u16,
    pub match_pool: Uint128,
}
