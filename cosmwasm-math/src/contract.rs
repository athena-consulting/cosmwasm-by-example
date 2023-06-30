#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{OperationsResponse, RESULT};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cosmwasm-math";
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
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Instantiating the state that will be stored to the blockchain */
    let operation_response = OperationsResponse {
        addition_result: 0,
        subtraction_result: 0,
        multiplication_result: 0,
        division_result: 0,
        modulo_result: 0,
        exponentiation_result: 0,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();
    // Save the stete in deps.storage which creates a storage for contract data on the blockchain.
    RESULT.save(deps.storage, &operation_response).unwrap();

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Operations { a, b } => execute::execute_operations(deps, a, b),
    }
}

pub mod execute {
    use super::*;

    pub fn execute_operations(deps: DepsMut, a: u128, b: u128) -> Result<Response, ContractError> {
        // Checking if numbers are not zero
        if a == 0 && b == 0 {
            return Err(ContractError::CanNotBeZero());
        }

        // Addition
        let addition_result = a + b;

        // Subtraction
        let subtraction_result = a - b;

        // Multiplication
        let multiplication_result = a * b;

        // Division
        let division_result = a / b;

        // Modulo
        let modulo_result = a % b;

        // Exponentiation
        let exponent: u32 = 3;
        let exponentiation_result: u128 = a.pow(exponent);

        // Create the response
        let response = OperationsResponse {
            addition_result,
            subtraction_result,
            multiplication_result,
            division_result,
            modulo_result,
            exponentiation_result,
        };

        // Fetching the state
        RESULT.load(deps.storage).unwrap();

        // Update the state
        RESULT.save(deps.storage, &response).unwrap();

        let res = Response::new().add_attributes(vec![
            ("action", "operations"),
            ("a", &a.to_string()),
            ("b", &b.to_string()),
            ("addition_res", &addition_result.to_string()),
            ("substraction_res", &subtraction_result.to_string()),
            ("multiplicationn_res", &multiplication_result.to_string()),
            ("division_res", &division_result.to_string()),
            ("modulo_res", &modulo_result.to_string()),
            ("exponential_res", &exponentiation_result.to_string()),
        ]);

        Ok(res)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {
        QueryMsg::GetResponse {} => query::get_response(deps),
    }
}

pub mod query {

    use super::*;

    pub fn get_response(deps: Deps) -> Result<QueryResponse, StdError> {
        let result = RESULT.load(deps.storage)?;

        to_binary(&result)
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

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetResponse {}).unwrap();
        let value: OperationsResponse = from_binary(&res).unwrap();

        assert_eq!(0, value.addition_result);
        assert_eq!(0, value.subtraction_result);
        assert_eq!(0, value.multiplication_result);
        assert_eq!(0, value.division_result);
        assert_eq!(0, value.modulo_result);
        assert_eq!(0, value.exponentiation_result);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // testing operation function
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Operations { a: 5, b: 5 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should get basic math operation for 5 and 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetResponse {  }).unwrap();
        let value: OperationsResponse = from_binary(&res).unwrap();
        assert_eq!(10, value.addition_result);
        assert_eq!(0, value.subtraction_result);
        assert_eq!(25, value.multiplication_result);
        assert_eq!(1, value.division_result);
        assert_eq!(0, value.modulo_result);
        assert_eq!(125, value.exponentiation_result);
    }
}
