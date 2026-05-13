use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateGift { recipient: String },
    Claim { gift_id: u64 },
    Cancel { gift_id: u64 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Gift { gift_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub denom: String,
    pub next_gift_id: u64,
}

#[cw_serde]
pub struct GiftResponse {
    pub gift_id: u64,
    pub creator: String,
    pub recipient: String,
    pub denom: String,
    pub amount: Uint128,
    pub status: String,
}
