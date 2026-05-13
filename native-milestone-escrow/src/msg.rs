use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

use crate::state::EscrowStatus;

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub owner: Option<String>,
    pub default_arbiter: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateEscrow {
        id: Option<u64>,
        seller: String,
        arbiter: Option<String>,
        amount: Uint128,
        description: String,
        timeout: Option<u64>,
    },
    Release {
        id: u64,
    },
    Refund {
        id: u64,
    },
    UpdateConfig {
        owner: Option<String>,
        default_arbiter: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(EscrowResponse)]
    Escrow { id: u64 },

    #[returns(NextEscrowIdResponse)]
    NextEscrowId {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
    pub denom: String,
    pub default_arbiter: Option<Addr>,
}

#[cw_serde]
pub struct EscrowResponse {
    pub id: u64,
    pub buyer: Addr,
    pub seller: Addr,
    pub arbiter: Addr,
    pub amount: Uint128,
    pub description: String,
    pub timeout: Option<u64>,
    pub created_at: u64,
    pub settled_at: Option<u64>,
    pub status: EscrowStatus,
}

#[cw_serde]
pub struct NextEscrowIdResponse {
    pub next_id: u64,
}
