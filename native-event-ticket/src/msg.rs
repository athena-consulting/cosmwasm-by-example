use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub organizer: String,
    pub denom: String,
    pub price: Uint128,
    pub max_tickets: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Buy {
        quantity: u64,
    },
    Withdraw {
        recipient: Option<String>,
        amount: Option<Uint128>,
    },
    UpdateOrganizer {
        organizer: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Tickets { buyer: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub organizer: String,
    pub denom: String,
    pub price: Uint128,
    pub max_tickets: u64,
    pub sold: u64,
    pub revenue: Uint128,
}

#[cw_serde]
pub struct TicketsResponse {
    pub buyer: String,
    pub quantity: u64,
}
