use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateBounty { title: String },
    Award { id: u64, recipient: String },
    Cancel { id: u64 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Bounty { id: u64 },
    NextId {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
}

#[cw_serde]
pub struct BountyResponse {
    pub id: u64,
    pub sponsor: String,
    pub title: String,
    pub amount: Uint128,
    pub settled: bool,
}

#[cw_serde]
pub struct NextIdResponse {
    pub next_id: u64,
}
