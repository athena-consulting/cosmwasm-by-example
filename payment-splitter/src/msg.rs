use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct MemberInput {
    pub address: String,
    pub share: u64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub members: Vec<MemberInput>,
    pub remainder_recipient: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Distribute {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(PreviewResponse)]
    Preview { amount: Uint128 },
}

#[cw_serde]
pub struct MemberResponse {
    pub address: String,
    pub share: u64,
}

#[cw_serde]
pub struct ConfigResponse {
    pub denom: String,
    pub members: Vec<MemberResponse>,
    pub total_shares: u128,
    pub remainder_recipient: String,
}

#[cw_serde]
pub struct CoinShare {
    pub address: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct PreviewResponse {
    pub shares: Vec<CoinShare>,
    pub remainder: Uint128,
    pub remainder_recipient: String,
}
