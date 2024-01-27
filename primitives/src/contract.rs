#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetOwnerResponse, InstantiateMsg, QueryMsg};
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
        .add_attribute("owner", info.sender)
        )
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query::owner(deps)?),
        QueryMsg::Integer {} => to_binary(&query::integers(deps)?)
    }
}

pub mod query {
    use cosmwasm_std::{Uint128, Uint64};

    use crate::msg::GetIntegerResponse;

    use super::*;

    pub fn owner(deps: Deps) -> StdResult<GetOwnerResponse> {
        // Load the state of the smart contract
        let state = STATE.load(deps.storage)?;
        Ok(GetOwnerResponse { owner: state.owner })
    }


    pub fn integers(_deps: Deps) -> StdResult<GetIntegerResponse> {
        /* Uint family in cosmwasm are thin wrappers around unsigned integers in Rust that uses strings for JSON encoding and decoding
        Uint64 is the smallest Uint among the group and
        The largest is Uint512
        
        Uints range from 0 to 2**(n-1) where n is the number of bytes for each different primitive type
        i.e Uint256 ranges from 0 to 2**255 */
        
        // verify Max of Uint128 
        // Use Uint::from() to convert from Rust primitive type to Cosmwasm Unsigned integer
        let _ex = Uint64::from(2u64);
        // Can return default value of cosmwasm primitives, in the case of Uint returns 0
        let _def = Uint128::default();
        let max_u128 = Uint128::from(340_282_366_920_938_463_463_374_607_431_768_211_455u128);
        let max_uint128_primitive = Uint128::MAX;
        assert_eq!(max_u128, max_uint128_primitive);

        /* Best practice is to avoid using too large unsigned integers in smart contracts
         for constants or numbers that should not exceed the max for the number. Always check for overflow (later example) */
        Ok(GetIntegerResponse { works: true })
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::GetIntegerResponse;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary};

    #[test]
    fn test_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("test_address", &[]);
        let instantiate_msg = InstantiateMsg {};
        instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();

        let query_msg = QueryMsg::GetOwner {  };
        let resp = query(deps.as_ref(),env.clone(), query_msg).unwrap();
        let owner_resp: GetOwnerResponse = from_binary(&resp).unwrap();
        assert_eq!(owner_resp.owner, "test_address")
    }

    #[test]
    fn test_integers() {
        let deps = mock_dependencies();
        let env = mock_env();
        let query_msg = QueryMsg::Integer {  };
        let resp = query(deps.as_ref(),env.clone(), query_msg).unwrap();
        let int_resp: GetIntegerResponse = from_binary(&resp).unwrap();
        assert_eq!(int_resp.works, true);
    }
    }

