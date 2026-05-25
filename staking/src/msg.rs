use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub token_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake {},
    Unstake { amount: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StakedResponse)]
    Staked { address: String },
}

#[cw_serde]
pub struct StakedResponse {
    pub amount: String,
}
