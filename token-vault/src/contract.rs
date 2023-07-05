#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError, Addr, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::*;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:token-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    /* Deps allows to access:
    1. Read/Write Storage Access
    2. General Blockchain APIs
    3. The Querier to the blototal_supplyckchain (raw data queries) */
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
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    /* Instantiating the state that will be stored to the blockchain */
   let total_supply=Uint128::zero();
   let token_info=TokenInfo{ token_denom: msg.token_symbol, token_address: msg.token_contract_address };
    // Save the stete in deps.storage which creates a storage for contract data on the blockchain.
    TOTAL_SUPPLY.save(deps.storage, &total_supply)?;
    TOKEN_INFO.save(deps.storage, &token_info)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("total_supply", total_supply))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {ExecuteMsg::Deposit{amount}=>execute::execute_deposit(deps,env,info,amount),
             ExecuteMsg::Withdraw { shares } => execute::execute_withdraw(deps,env,info,shares), }
}
pub mod execute {
    use cosmwasm_std::{CosmosMsg, WasmQuery};

    use super::*;

    pub fn execute_deposit(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let token_info=TOKEN_INFO.load(deps.storage)?;
        let mut total_supply=TOTAL_SUPPLY.load(deps.storage)?;
        let mut shares=Uint128::zero();
        let mut balance=BALANCE_OF.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        let balance_of=get_token_balance_of(&deps, info.sender.clone(), token_info.token_address.clone())?;
        if total_supply.is_zero() {
            shares+=amount;
        }
        else {
            shares+=amount.checked_mul(total_supply).map_err(StdError::overflow)?.checked_div(balance_of).map_err(StdError::divide_by_zero)?
        }

        give_allowance(env.clone(), info.clone(), amount, token_info.token_address.clone())?;
        total_supply+=shares;
        TOTAL_SUPPLY.save(deps.storage, &total_supply)?;
        balance+=shares;
        BALANCE_OF.save(deps.storage, info.sender.clone(), &balance)?;

        let transfer_from_msg=cw20::Cw20ExecuteMsg::TransferFrom { owner: info.sender.to_string(), recipient: env.contract.address.to_string(), amount };
        let msg=CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute { contract_addr: token_info.token_address.to_string(), msg: to_binary(&transfer_from_msg)?, funds: info.funds });

        Ok(Response::new().add_attribute("action", "deposit").add_message(msg))

        
    }

    pub fn execute_withdraw(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        shares: Uint128,
    ) -> Result<Response, ContractError> {
        let token_info=TOKEN_INFO.load(deps.storage)?;
        let mut total_supply=TOTAL_SUPPLY.load(deps.storage)?;
        let mut balance=BALANCE_OF.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        let balance_of=get_token_balance_of(&deps, info.sender.clone(), token_info.token_address.clone())?;
        let amount=shares.checked_mul(balance_of).map_err(StdError::overflow)?.checked_div(total_supply).map_err(StdError::divide_by_zero)?;
        total_supply-=shares;
        TOTAL_SUPPLY.save(deps.storage, &total_supply)?;
        balance-=shares;
        BALANCE_OF.save(deps.storage, info.sender.clone(), &balance)?;

        let transfer_msg=cw20::Cw20ExecuteMsg::Transfer { recipient: info.sender.to_string(), amount};
        let msg=CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute { contract_addr: token_info.token_address.to_string(), msg: to_binary(&transfer_msg)?, funds: info.funds });

        Ok(Response::new().add_attribute("action", "withdraw").add_message(msg))

        
    }
    
  
    pub fn get_token_balance_of(
        deps: &DepsMut,
        user_address: Addr,
        cw20_contract_addr: Addr,
    ) -> Result<Uint128, ContractError> {
        let query_msg=cw20::Cw20QueryMsg::Balance { address: user_address.to_string() };
       let msg=deps.querier.query(&cosmwasm_std::QueryRequest::Wasm(WasmQuery::Smart { contract_addr: cw20_contract_addr.to_string(), msg: to_binary(&query_msg)? }))?;
    
        Ok(msg)
    }

    pub fn give_allowance(
        env: Env,
        info: MessageInfo,
        amount: Uint128,
        cw20_contract_addr: Addr,
    ) -> Result<Response, ContractError> {
        let allowance_msg=cw20::Cw20ExecuteMsg::IncreaseAllowance { spender: env.contract.address.to_string(), amount , expires: None };
       let msg=CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute { contract_addr: cw20_contract_addr.to_string(), msg: to_binary(&allowance_msg)?, funds: info.funds });
    
        Ok(Response::new().add_attribute("action", "give_Allowance").add_message(msg))
    }
    
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {QueryMsg::GetTotalSupply{}=>query::get_total_supply(deps),
    QueryMsg::GetBalanceOf { address } => query::get_balance_of(deps,address) }
}

pub mod query {

    use super::*;

    pub fn get_total_supply(deps: Deps) -> Result<QueryResponse, StdError> {
        let total_supply = TOTAL_SUPPLY.load(deps.storage)?;
    
        to_binary(&total_supply)
    }

    pub fn get_balance_of(deps: Deps,addr: Addr) -> Result<QueryResponse, StdError> {
        let balance_of = BALANCE_OF.load(deps.storage,addr)?;
    
        to_binary(&balance_of)
    }
    
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, coins, Uint128, Addr, StdError};

    use crate::{msg::{InstantiateMsg, ExecuteMsg}, contract::{instantiate,execute,}, ContractError};



    #[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg { token_symbol: "ABC".to_string(), token_contract_address: Addr::unchecked("abcdef") };
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

       // Assert the response contains the expected attributes
       let response = res.unwrap();
       assert_eq!(response.attributes.len(), 2);
       assert_eq!(response.attributes[0].key, "method");
       assert_eq!(response.attributes[0].value, "instantiate");
       assert_eq!(response.attributes[1].key, "total_supply");
       assert_eq!(response.attributes[1].value, Uint128::zero().to_string());
}

#[test]
fn test_execute_receive() {
    let mut deps = mock_dependencies();
    let info = mock_info("sender", &[]);

    
    let msg = InstantiateMsg { token_symbol: "ABC".to_string(), token_contract_address: Addr::unchecked("abcdef") };
    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);
    assert!(res.is_ok());

    // Assert the response contains the expected attributes
    let response = res.unwrap();
    assert_eq!(response.attributes.len(), 2);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "instantiate");
    assert_eq!(response.attributes[1].key, "total_supply");
    assert_eq!(response.attributes[1].value, Uint128::zero().to_string());

    let msg=ExecuteMsg::Deposit { amount: Uint128::new(10) };
    let err=execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();


    assert_eq!(err,
       ContractError::Std(StdError::GenericErr {msg: "Querier system error: No such contract: abcdef".to_string()}));
    
}

#[test]
fn test_execute_withdraw() {
    let mut deps = mock_dependencies();
    let info = mock_info("sender", &[]);

    
    let msg = InstantiateMsg { token_symbol: "ABC".to_string(), token_contract_address: Addr::unchecked("abcdef") };
    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg);
    assert!(res.is_ok());

    // Assert the response contains the expected attributes
    let response = res.unwrap();
    assert_eq!(response.attributes.len(), 2);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "instantiate");
    assert_eq!(response.attributes[1].key, "total_supply");
    assert_eq!(response.attributes[1].value, Uint128::zero().to_string());

    let msg=ExecuteMsg::Withdraw { shares: Uint128::new(10) };
    let err=execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();


    assert_eq!(err,
       ContractError::Std(StdError::GenericErr {msg: "Querier system error: No such contract: abcdef".to_string()}));
    
}
}

