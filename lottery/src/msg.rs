use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub ticket_price: String,
    pub ticket_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    BuyTicket {},
    DrawWinner {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(WinnerResponse)]
    Winner {},
}

#[cw_serde]
pub struct WinnerResponse {
    pub winner: String,
}
