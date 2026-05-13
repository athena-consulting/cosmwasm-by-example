use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateInvoice {
        payer: Option<String>,
        amount: Uint128,
        memo: Option<String>,
    },
    PayInvoice {
        invoice_id: u64,
    },
    CancelInvoice {
        invoice_id: u64,
    },
    Withdraw {
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Invoice { invoice_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub balance: Uint128,
    pub next_invoice_id: u64,
}

#[cw_serde]
pub struct InvoiceResponse {
    pub invoice_id: u64,
    pub payer: Option<String>,
    pub amount: Uint128,
    pub memo: Option<String>,
    pub paid: bool,
    pub paid_by: Option<String>,
}
