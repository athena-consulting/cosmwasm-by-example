#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, HelloWorldResponse};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:hello-world";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HelloWorld{} => to_binary(&query_hello_world()?),
    }
    
}

pub fn query_hello_world() -> StdResult<HelloWorldResponse> {
    // Sets the string in the struct to `HelloWorldResponse` and returns it as response to query
    let hello_message = HelloWorldResponse {hello_world_message: "Hello World".to_string()};
    Ok(hello_message)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::{mock_dependencies, mock_env}, from_binary};

    use super::*;



    #[test]
    fn basic_hello() {
        /* Testing that query works. */
        let deps = mock_dependencies();
        let env = mock_env();
        let msg = QueryMsg::HelloWorld{};
        let q = query(deps.as_ref(), env, msg);
        let res: HelloWorldResponse = from_binary(&q.unwrap()).unwrap();
        assert_eq!(res.hello_world_message, "Hello World")
    }
}
