use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};



// Total Supply
pub const TOTAL_SUPPLY: Item<Uint128> = Item::new("total_supply");

// Balance of
pub const BALANCE_OF: Map<Addr,Uint128>=Map::new("balance_of");


#[cw_serde]
pub struct  TokenInfo{
    pub token_denom: String,
    pub token_address: Addr

}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
