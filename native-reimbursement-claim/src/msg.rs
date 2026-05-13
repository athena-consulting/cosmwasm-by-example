use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Fund {},
    ApproveClaim {
        claimant: String,
        amount: Uint128,
        description: Option<String>,
    },
    Claim {
        claim_id: u64,
    },
    CancelClaim {
        claim_id: u64,
    },
    WithdrawExcess {
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Claim { claim_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub balance: Uint128,
    pub next_claim_id: u64,
}

#[cw_serde]
pub struct ClaimResponse {
    pub claim_id: u64,
    pub claimant: String,
    pub amount: Uint128,
    pub description: Option<String>,
    pub paid: bool,
}
