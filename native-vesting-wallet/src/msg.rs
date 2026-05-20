use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub beneficiary: String,
    pub start_height: u64,
    pub cliff_height: u64,
    pub end_height: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VestingResponse)]
    Vesting {},
}

#[cw_serde]
pub struct VestingResponse {
    pub creator: String,
    pub beneficiary: String,
    pub denom: String,
    pub total: Uint128,
    pub claimed: Uint128,
    pub vested: Uint128,
    pub claimable: Uint128,
    pub start_height: u64,
    pub cliff_height: u64,
    pub end_height: u64,
}
