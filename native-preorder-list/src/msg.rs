use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub item: String,
    pub unit_price: Uint128,
    pub max_units: Option<u64>,
}

#[cw_serde]
pub enum ExecuteMsg {
    PlaceOrder {
        quantity: u64,
        recipient: Option<String>,
        note: Option<String>,
    },
    MarkFulfilled {
        order_id: u64,
    },
    RefundOrder {
        order_id: u64,
    },
    Withdraw {
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Order { order_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub item: String,
    pub unit_price: Uint128,
    pub max_units: Option<u64>,
    pub sold_units: u64,
    pub balance: Uint128,
    pub next_order_id: u64,
}

#[cw_serde]
pub struct OrderResponse {
    pub order_id: u64,
    pub buyer: String,
    pub recipient: String,
    pub quantity: u64,
    pub paid: Uint128,
    pub note: Option<String>,
    pub fulfilled: bool,
    pub refunded: bool,
}
