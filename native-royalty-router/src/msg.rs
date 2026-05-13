use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub shares: Vec<ShareInput>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RoutePayment {},
    UpdateShares { shares: Vec<ShareInput> },
    UpdateOwner { owner: String },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

#[cw_serde]
pub struct ShareInput {
    pub recipient: String,
    pub bps: u16,
}

#[cw_serde]
pub struct ShareResponse {
    pub recipient: String,
    pub bps: u16,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub shares: Vec<ShareResponse>,
}
