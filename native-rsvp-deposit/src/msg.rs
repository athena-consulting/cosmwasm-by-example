use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub forfeiture_recipient: String,
    pub denom: String,
    pub deposit_amount: Uint128,
    pub event_time: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Reserve {},
    CheckIn { attendee: String },
    Refund {},
    Forfeit { attendee: String },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Attendee { attendee: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub forfeiture_recipient: String,
    pub denom: String,
    pub deposit_amount: Uint128,
    pub event_time: u64,
}

#[cw_serde]
pub struct AttendeeResponse {
    pub attendee: String,
    pub reserved: bool,
    pub checked_in: bool,
    pub settled: bool,
}
