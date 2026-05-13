use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub merchant: Option<String>,
    pub default_recipient: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateInvoice {
        id: Option<u64>,
        payer: Option<String>,
        recipient: Option<String>,
        amount: Uint128,
        description: String,
        due_at: Option<u64>,
    },
    PayInvoice {
        id: u64,
    },
    CancelInvoice {
        id: u64,
    },
    UpdateConfig {
        merchant: Option<String>,
        default_recipient: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(InvoiceResponse)]
    Invoice { id: u64 },

    #[returns(NextInvoiceIdResponse)]
    NextInvoiceId {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub merchant: Addr,
    pub default_recipient: Addr,
    pub denom: String,
}

#[cw_serde]
pub struct InvoiceResponse {
    pub id: u64,
    pub payer: Option<Addr>,
    pub recipient: Addr,
    pub amount: Uint128,
    pub description: String,
    pub due_at: Option<u64>,
    pub paid: bool,
    pub canceled: bool,
    pub paid_at: Option<u64>,
}

#[cw_serde]
pub struct NextInvoiceIdResponse {
    pub next_id: u64,
}
