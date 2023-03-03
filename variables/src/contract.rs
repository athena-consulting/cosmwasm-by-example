#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:variables";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    /* Deps allows to access:
    1. Read/Write Storage Access
    2. General Blockchain APIs
    3. The Querier to the blockchain (raw data queries) */
    deps: DepsMut,
    /* env gives access to global variables which represent environment information.
    For exaample: 
    - Block Time/Height
    - contract address
    - Transaction Info */
    _env: Env,
    /* Message Info gives access to information used for authorization.
    1. Funds sent with the message.
    2. The message sender (signer). */
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Instantiating the state that will be stored to the blockchain */
    let state = State {
        count: msg.count,
        /* info.sender is the address of the signer of the message and is stored in this instance in storage as the owner of the contract
        (which is used for access control to smart contract functions) */
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Save the stete in deps.storage which creates a storage for contract data on the blockchain. 
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
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
        ExecuteMsg::Increment {} => execute::increment(deps),
    }
}

pub mod execute {
    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        /* Changes a state variable which changes the count variable on the blockchain. */
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    }


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
        QueryMsg::GetBlock {} => to_binary(&query::block(deps,env)?),

    }
}

pub mod query {
    use crate::msg::GetBlockResponse;

    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        /* Queries data from the blockchain storage and returns the count */
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }

    pub fn block(_deps: Deps, env: Env) -> StdResult<GetBlockResponse> {
        /* Queries global variables from environment */ 
        let height = env.block.height;
        let time = env.block.time.seconds();
        Ok(GetBlockResponse { height: height, time: time })
    }
}

#[cfg(test)]
mod tests {
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
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

}
