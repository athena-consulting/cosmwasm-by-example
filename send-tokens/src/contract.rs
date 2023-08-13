#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};




#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SendTokens {amount, denom, to} => execute::send_tokens(deps, amount, denom, to),
    }
}

pub mod execute {
    use cosmwasm_std::{Uint128, Addr, BankMsg, Coin};

    use super::*;

    pub fn send_tokens(_deps: DepsMut, amount: Uint128, denom: String, to: Addr) -> Result<Response, ContractError> {

        Ok(Response::new().add_attribute("action", "increment")
        /* Sending tokens is part of the response of a function
        Developer creates a BankMsg to send tokens to an address with a specific native token
        Will fail if smart contract does not have this much tokens initially  */
        .add_message(BankMsg::Send { to_address: to.into_string(), amount: vec![Coin{denom, amount}] }))
    }

   
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
    }
}

pub mod query {

   
}

#[cfg(test)]
mod tests {
}
