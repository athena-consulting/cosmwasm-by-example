#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, NAMES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:read-write-state";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Write {} => execute::write(deps),
    }
}

pub mod execute {
    use super::*;

    pub fn write(deps: DepsMut) -> Result<Response, ContractError> {
        /* This execute endpoint writes a new owner to state. */
        
        // First we need to load the current state from the blockchain from `deps.storage` as mutable.
        let mut state = STATE.load(deps.storage)?;
        state.count = 5;
        
        // Save the new state with the changed variables in storage.
        STATE.save(deps.storage, &state)?;

        // Now let us add a new key-value pair to the `NAMES` map in storage. 
        NAMES.save(deps.storage, "Georges".to_string(), &"Chouchani".to_string())?;
        
        Ok(Response::new().add_attribute("action", "write"))
    }

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
        QueryMsg::GetFamilyName {first_name} => to_binary(&query::name(deps, first_name)?),

    }
}

pub mod query {
    use crate::msg::GetNameResponse;

    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        // Loads the state from storage and checks the count variable.
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }

    pub fn name(deps: Deps, first_name: String) -> StdResult<GetNameResponse> {
        // Loads the NAMES Map from storage for the key `first_name` to get the `last_name`
        // `may_load` returns None if the key does not exist in the map. `load` returns an error.
        let res = NAMES.may_load(deps.storage, first_name)?;
        Ok(GetNameResponse{family_name: res.unwrap()})
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::GetNameResponse;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn write() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Execute the function that changes the state of the blockchain.
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Write {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // The counter should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();

        assert_eq!(5, value.count);

        // Last name of Georges can now be found. 
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetFamilyName { first_name: "Georges".to_string() } ).unwrap();
        let value: GetNameResponse = from_binary(&res).unwrap();

        assert_eq!(value.family_name, "Chouchani".to_string())
    }

}
