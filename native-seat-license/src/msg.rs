use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub seat_price: Uint128,
    pub max_seats: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    BuySeat { account: Option<String> },
    TransferSeat { recipient: String },
    RevokeSeat { account: String },
    Withdraw { recipient: String, amount: Uint128 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Seat { account: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub seat_price: Uint128,
    pub max_seats: u64,
    pub sold_seats: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct SeatResponse {
    pub account: String,
    pub has_seat: bool,
}
