#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError, Addr,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::*;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:recieve-cw20-tokens";
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
    let admin = info.sender.to_string();

    // contract address with denom that will be whitelisted to send toke to this address
    let cw20_whitelist: Vec<(String, Addr)> = vec![(
        msg.token_symbol,
        msg.token_contract_address,
    )];

    // Save the stete in deps.storage which creates a storage for contract data on the blockchain.
    CONFIG.save(
        deps.storage,
        &Config {
            admin: deps.api.addr_validate(&admin).unwrap(),
            cw20_wl: cw20_whitelist,
        },
    )?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // cw20 receive wrapper
        ExecuteMsg::Receive(receive_msg) =>execute::execute_receive(deps, env, info, receive_msg),
    }
}
pub mod execute {
    use cosmwasm_std::from_binary;
    use cw20::{Cw20ReceiveMsg, Cw20CoinVerified, Balance};

    use crate::msg::ReceiveMsg;

    use super::*;

    pub fn execute_receive(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        wrapper: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        // Message included in Send{contract, amount, **msg**} execute on the cw20 contract
        let msg: ReceiveMsg = from_binary(&wrapper.msg).unwrap();
    
        // Address that executed the "Send" on the cw20 contract
        let user_wallet = deps.api.addr_validate(&wrapper.sender).unwrap();
    
        // Constructing cw20 balance
        let balance = Balance::Cw20(Cw20CoinVerified {
            // cw20 contract this message was sent from
            address: info.sender.clone(),
            // Send{contract, **amount**, msg}
            amount: wrapper.amount,
        });
    
        // Load config for whitelist check
        let config = CONFIG.load(deps.storage)?;
    
        // Check constructed cw20 balance , returns contract error if not
        is_balance_whitelisted(&balance, &config)?;
    
        match msg {
            // Message included in the "Send{contract, amount, **msg**}" call on the cw20 contract,
            ReceiveMsg::AnExecuteMsg {} => {
                execute_do_something(deps, &user_wallet, &info.sender, balance)
            }
        }
    }
    
    pub fn execute_do_something(
        _deps: DepsMut,
        _user_wallet: &Addr,
        _cw20_contract_addr: &Addr,
        _balance: Balance,
    ) -> Result<Response, ContractError> {
        // insert your execution logic here
    
        Ok(Response::default())
    }
    
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {
        QueryMsg::GetAdmin {} =>query::get_admin(deps),
    }
}

pub mod query {

    use crate::msg::AdminResponse;

    use super::*;

    pub fn get_admin(deps: Deps) -> Result<QueryResponse, StdError> {
        let config = CONFIG.load(deps.storage)?;
    
        let admin = config.admin.to_string();
    
        to_binary(&AdminResponse { admin })
    }
    
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, coins, to_binary, Uint128, Addr, from_binary};
    use cw20::Cw20ReceiveMsg;

    use crate::{msg::{InstantiateMsg, ReceiveMsg, ExecuteMsg, QueryMsg, AdminResponse}, contract::{instantiate,execute,query}, state::{Config, CONFIG}};


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
    assert_eq!(response.attributes[1].key, "admin");
    assert_eq!(response.attributes[1].value, "creator");
}

#[test]
fn test_execute_receive() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let msg = InstantiateMsg { token_symbol: "ABC".to_string(), token_contract_address: Addr::unchecked("abcdef") };
    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_ok());

    // Assert the response contains the expected attributes
    let response = res.unwrap();
    assert_eq!(response.attributes.len(), 2);
    assert_eq!(response.attributes[0].key, "method");
    assert_eq!(response.attributes[0].value, "instantiate");
    assert_eq!(response.attributes[1].key, "admin");
    assert_eq!(response.attributes[1].value, "sender");



    let wrapper = Cw20ReceiveMsg {
        sender: "abcdef".to_string(),
        amount: Uint128::new(100),
        msg: to_binary(&ReceiveMsg::AnExecuteMsg {}).unwrap(),
    };
    let msg=ExecuteMsg::Receive(wrapper);

    // As recieve will be directly called by cw20 contract so in info we will have cw20 address stored rather than an user address
    // abcdef is cw20 address
    let info = mock_info("abcdef", &[]);

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0,res.messages.len());

}

#[test]
fn test_query_get_admin() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let msg = QueryMsg::GetAdmin {};

    // Set the expected admin value
    let expected_admin = "admin_address".to_string();

    // Set the config in storage
    let config = Config {
        admin: Addr::unchecked(expected_admin.clone()),
        cw20_wl: vec![],
    };
    CONFIG.save(deps.as_mut().storage, &config).unwrap();

    let res = query(deps.as_ref(), env, msg);
    assert!(res.is_ok());

    // Assert the admin value is the expected value
    let response = res.unwrap();
    let admin_response: AdminResponse = from_binary(&response).unwrap();
    assert_eq!(admin_response.admin, expected_admin);
}
}

