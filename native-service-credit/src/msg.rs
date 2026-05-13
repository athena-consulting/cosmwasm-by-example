use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub unit_price: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    BuyCredits {
        account: Option<String>,
        credits: u64,
    },
    TransferCredits {
        recipient: String,
        credits: u64,
    },
    DebitCredits {
        account: String,
        credits: u64,
        reason: Option<String>,
    },
    Withdraw {
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Credit { account: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub unit_price: Uint128,
    pub balance: Uint128,
}

#[cw_serde]
pub struct CreditResponse {
    pub account: String,
    pub credits: u64,
}
