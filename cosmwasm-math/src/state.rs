use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct OperationsResponse {
    pub addition_result: u128,
    pub subtraction_result: u128,
    pub multiplication_result: u128,
    pub division_result: u128,
    pub modulo_result: u128,
    pub exponentiation_result: u128,
}
// Mapping of result of two numbers

pub const RESULT: Item<OperationsResponse> = Item::new("result");
