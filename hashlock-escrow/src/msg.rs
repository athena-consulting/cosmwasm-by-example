use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {
    pub recipient: String,
    pub hash: Binary,
    pub deadline: Timestamp,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim { preimage: Binary },
    Refund {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(EscrowResponse)]
    Escrow {},
}

#[cw_serde]
pub struct EscrowResponse {
    pub sender: String,
    pub recipient: String,
    pub hash: Binary,
    pub deadline: Timestamp,
    pub amount: Coin,
    pub claimed: bool,
    pub refunded: bool,
}

#[cw_serde]
pub struct MigrateMsg {}
