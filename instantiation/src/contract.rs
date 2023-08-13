#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:instantiation";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Code is executed at contract creation and accepts an `InstantiateMsg`
    Used to set initial (default) state variables of the smart contract
    Can perform checks and acts like an execute message.
     */
    let state = State {
        global_var: msg.sent_message,
    };
    // Pushes the data to blockchain storage
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
    .add_attribute("instantiated", "true"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMessage {} => to_binary(&query::message(deps)?),
    }

}

pub mod query {

    use crate::msg::GetMessageResponse;

    use super::*;

    pub fn message(deps: Deps) -> StdResult<GetMessageResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetMessageResponse { message: state.global_var })
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, Addr, from_binary};

    use crate::msg::GetMessageResponse;

    use super::*;

    #[test]
    fn instantiate_test() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let msg = InstantiateMsg { sent_message: "Hey!".to_string()};
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query data from blockchain
        let res = query(deps.as_ref(), env, QueryMsg::GetMessage {  }).unwrap();
        let message: GetMessageResponse = from_binary(&res).unwrap();
        assert_eq!("Hey!".to_string(), message.message);
    }   
}
