use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub claim_amount: Uint128,
    pub cooldown_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {},
    Fund {},
    UpdateConfig {
        owner: Option<String>,
        claim_amount: Option<Uint128>,
        cooldown_seconds: Option<u64>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(LastClaimResponse)]
    LastClaim { address: String },
    #[returns(NextClaimResponse)]
    NextClaim { address: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub claim_amount: Uint128,
    pub cooldown_seconds: u64,
}

#[cw_serde]
pub struct LastClaimResponse {
    pub address: String,
    pub last_claim_at: Option<u64>,
}

#[cw_serde]
pub struct NextClaimResponse {
    pub address: String,
    pub next_claim_at: u64,
    pub can_claim: bool,
}
