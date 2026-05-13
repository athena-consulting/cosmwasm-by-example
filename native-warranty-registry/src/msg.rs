use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
    pub registration_fee: Uint128,
    pub coverage_seconds: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterProduct { serial: String, metadata: String },
    TransferProduct { serial: String, new_owner: String },
    FileClaim { serial: String, note: String },
    ResolveClaim { claim_id: u64, approved: bool },
    Withdraw { recipient: String, amount: Uint128 },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Product { serial: String },
    Claim { claim_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub registration_fee: Uint128,
    pub coverage_seconds: u64,
    pub next_claim_id: u64,
    pub balance: Uint128,
}

#[cw_serde]
pub struct ProductResponse {
    pub serial: String,
    pub owner: Option<String>,
    pub metadata: Option<String>,
    pub expires_at: Option<u64>,
    pub is_expired: bool,
}

#[cw_serde]
pub struct ClaimResponse {
    pub id: u64,
    pub serial: String,
    pub claimant: String,
    pub note: String,
    pub status: String,
}
