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
    CreateBudget {
        manager: String,
        label: String,
        cap: Uint128,
    },
    ClaimBudget {
        budget_id: u64,
        recipient: Option<String>,
        amount: Uint128,
        memo: Option<String>,
    },
    CloseBudget {
        budget_id: u64,
    },
    WithdrawUnreserved {
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Budget { budget_id: u64 },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub denom: String,
    pub next_budget_id: u64,
    pub balance: Uint128,
    pub reserved: Uint128,
    pub unreserved: Uint128,
}

#[cw_serde]
pub struct BudgetResponse {
    pub id: u64,
    pub manager: String,
    pub label: String,
    pub cap: Uint128,
    pub spent: Uint128,
    pub remaining: Uint128,
    pub active: bool,
}
