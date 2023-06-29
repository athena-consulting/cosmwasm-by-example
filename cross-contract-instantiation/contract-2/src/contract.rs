#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Instantiate {} => try_instantiate(deps, info),
    }
}

pub fn try_instantiate(_deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    // Creating instantiate message of contract 1 from this contract
    let instantiation_msg = contract_1::msg::InstantiateMsg { count: 1 };

    // Instantiating the contract 1
    let msgs = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
        admin: None,
        code_id: 0,
        msg: to_binary(&instantiation_msg)?,
        funds: info.funds,
        label: "Cross_Contract_instantiation".to_string(),
    });

    let res = Response::new()
        .add_attribute("method", "instantiate_another")
        .add_message(msgs);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn cross_contract_instantiation() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // INstantiating another contract
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Instantiate {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Set the desired instantiate message
        let instantiation_msg = contract_1::msg::InstantiateMsg { count: 1 };

        // Set the expected CosmosMsg for contract instantiation
        let expected_msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Instantiate {
            admin: None,
            code_id: 0,
            msg: to_binary(&instantiation_msg).unwrap(),
            funds: info.funds,
            label: "Cross_Contract_instantiation".to_string(),
        });
        // Assert the expected response and attributes
        let expected_res = Response::new()
            .add_attribute("method", "instantiate_another")
            .add_message(expected_msg);
        assert_eq!(_res, expected_res);
    }
}
