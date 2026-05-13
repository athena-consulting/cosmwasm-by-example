use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub merchant: String,
    pub denom: String,
    pub referral_bps: u16,
}

#[cw_serde]
pub enum ExecuteMsg {
    Pay {
        referrer: Option<String>,
    },
    UpdateConfig {
        merchant: Option<String>,
        referral_bps: Option<u16>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub merchant: String,
    pub denom: String,
    pub referral_bps: u16,
}
