use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Tip {},
    Withdraw {
        amount: Option<Uint128>,
        recipient: Option<String>,
    },
    UpdateOwner {
        owner: String,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Balance {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
}

#[cw_serde]
pub struct BalanceResponse {
    pub denom: String,
    pub amount: Uint128,
}
