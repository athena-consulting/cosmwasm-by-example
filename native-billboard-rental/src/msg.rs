use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub slot_price: Uint128,
    pub rental_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateSlot { label: String },
    RentSlot { slot_id: u64, message: String },
    UpdateMessage { slot_id: u64, message: String },
    CloseSlot { slot_id: u64 },
    Withdraw { recipient: String, amount: Uint128 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Slot { slot_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub slot_price: Uint128,
    pub rental_seconds: u64,
    pub next_slot_id: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct SlotResponse {
    pub id: u64,
    pub label: String,
    pub active: bool,
    pub renter: Option<String>,
    pub message: Option<String>,
    pub paid_until: u64,
    pub is_available: bool,
}
