use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub denom: String,
    pub deposit_amount: Uint128,
    pub refund_deadline: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit {},
    Refund {},
    Forfeit {
        depositor: String,
        recipient: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(DepositResponse)]
    Deposit { address: String },
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub denom: String,
    pub deposit_amount: Uint128,
    pub refund_deadline: u64,
}

#[cw_serde]
pub struct DepositResponse {
    pub address: String,
    pub has_deposit: bool,
    pub amount: Uint128,
    pub deposited_at: Option<u64>,
}
