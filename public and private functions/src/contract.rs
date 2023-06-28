#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:primitives";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Saves the state of the smart contract from the Instantiate Msg */
    let state = State {
        /* Info.sender is a global function variable that explains who is the signer of a message. */
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::PublicFunction { param } => execute::public_function(deps, env, info, param),
    }
}

pub mod execute {
    use super::*;

    // Public function that can be called externally
    pub fn public_function(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        param: String,
    ) -> Result<Response, ContractError> {
        // Perform the desired logic
        let result = do_something(param);

        // Return a response
        let response = Response::new().add_attribute("result", result);

        Ok(response)
    }

    // Private function that can only be called internally
    fn do_something(param: String) -> String {
        // Perform some internal logic
        let result = format!("Doing something with param: {}", param);

        result
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn test_public_function() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("test_address", &[]);
        let instansiate_msg = InstantiateMsg {};
        instantiate(deps.as_mut(), env.clone(), info.clone(), instansiate_msg).unwrap();

        // Calling public function which will call the private function and getting the result from private function
        let msg = ExecuteMsg::PublicFunction {
            param: "Hello cosmwasm".to_string(),
        };
        let resp = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(
            resp,
            Response::new().add_attribute("result", "Doing something with param: Hello cosmwasm")
        );
    }
}
