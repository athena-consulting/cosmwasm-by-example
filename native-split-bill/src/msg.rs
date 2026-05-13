use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateBill {
        recipient: String,
        participants: Vec<String>,
        share_amount: Uint128,
        memo: Option<String>,
    },
    PayShare {
        bill_id: u64,
    },
    Settle {
        bill_id: u64,
    },
    Cancel {
        bill_id: u64,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Bill { bill_id: u64 },
    ShareStatus { bill_id: u64, participant: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub next_bill_id: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct BillResponse {
    pub id: u64,
    pub creator: String,
    pub recipient: String,
    pub participants: Vec<String>,
    pub share_amount: Uint128,
    pub total_amount: Uint128,
    pub paid_count: u64,
    pub status: String,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct ShareStatusResponse {
    pub bill_id: u64,
    pub participant: String,
    pub paid: bool,
}
