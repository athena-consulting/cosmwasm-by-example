use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub token_symbol: String,
    pub token_contract_address: Addr,
}


#[cw_serde]
pub enum ExecuteMsg {

    Deposit {
        amount : Uint128
    },
    Withdraw {
        shares: Uint128
    }
}



///~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
////// Query
///~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint128)]
    GetTotalSupply {},
    
    #[returns(Uint128)]
    GetBalanceOf {
        address: Addr
    }
}


