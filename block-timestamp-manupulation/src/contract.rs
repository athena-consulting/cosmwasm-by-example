#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, CosmosMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::PREVIOUS_BLOCK_TIME;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:block-timestmp-manupulation";
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
    env: Env,
    /* Message Info gives access to information used for authorization.
    1. Funds sent with the message.
    2. The message sender (signer). */
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let prev_block_time = 0;
    PREVIOUS_BLOCK_TIME
        .save(deps.storage, &prev_block_time)
        .unwrap();
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION).unwrap();

    let msg=CosmosMsg::Bank(cosmwasm_std::BankMsg::Send { to_address: env.contract.address.to_string(), amount: info.funds });

    Ok(Response::new().add_attribute("method", "instantiate").add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Guess {} => execute::execute_guess(deps, env, info),
    }
}

pub mod execute {
    use cosmwasm_std::{coin, Addr, CosmosMsg};

    use super::*;

    pub fn execute_guess(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let contract_address=env.contract.address.to_string();
        let mut prev_block_time = PREVIOUS_BLOCK_TIME.load(deps.storage).unwrap();
        let required_fund=vec![coin(2, "earth")];
        if !(info.funds == required_fund ) {
            return Err(ContractError::InsufficientFunds());
        }

        if env.block.time.seconds() == prev_block_time {
            return Err(ContractError::BlockTimestampError());
        }

        prev_block_time = env.block.time.seconds();
        PREVIOUS_BLOCK_TIME
            .save(deps.storage, &prev_block_time)
            .unwrap();

        if env.block.time.seconds() % 7 == 0 {
            send_all_funds(deps, env, info.sender)?;
        }
        let msg=CosmosMsg::Bank(cosmwasm_std::BankMsg::Send { to_address: contract_address, amount: info.funds });

        Ok(Response::new().add_attribute("action", "guess").add_message(msg))
    }

    pub fn send_all_funds(deps: DepsMut, env: Env, to: Addr) -> Result<Response, ContractError> {
        let contract_balance = deps
            .querier
            .query_balance(env.contract.address, "earth")
            .unwrap()
            .amount;
        let msg = CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: to.to_string(),
            amount: vec![coin(contract_balance.into(), "earth".to_string())],
        });
        Ok(Response::new()
            .add_attribute("action", "send_all_funds")
            .add_message(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn guess() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());

        let info_1 = mock_info("creator", &coins(2, "earth"));

        let msg=ExecuteMsg::Guess {  };
        let res=execute(deps.as_mut(),mock_env(), info_1, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
