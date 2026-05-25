#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        arbiter: deps.api.addr_validate(&msg.arbiter)?,
        recipient: deps.api.addr_validate(&msg.recipient)?,
        source: info.sender,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Approve {} => execute_approve(deps, env, info),
        ExecuteMsg::Refund {} => execute_refund(deps, env, info),
    }
}

pub fn execute_approve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.arbiter {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps.querier.query_all_balances(&env.contract.address)?;
    
    let msg = BankMsg::Send {
        to_address: config.recipient.into_string(),
        amount: balance,
    };

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_message(msg))
}

pub fn execute_refund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.arbiter {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps.querier.query_all_balances(&env.contract.address)?;
    
    let msg = BankMsg::Send {
        to_address: config.source.into_string(),
        amount: balance,
    };

    Ok(Response::new()
        .add_attribute("action", "refund")
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        arbiter: config.arbiter.into_string(),
        recipient: config.recipient.into_string(),
        source: config.source.into_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, CosmosMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            arbiter: "arbiter".to_string(),
            recipient: "recipient".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn approve() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            arbiter: "arbiter".to_string(),
            recipient: "recipient".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // only arbiter can approve
        let info = mock_info("creator", &[]);
        let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Approve {}).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // arbiter approves
        let info = mock_info("arbiter", &[]);
        deps.querier.update_balance(mock_env().contract.address, coins(1000, "earth"));

        let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Approve {}).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "recipient".to_string(),
                amount: coins(1000, "earth"),
            })
        );
    }

    #[test]
    fn refund() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            arbiter: "arbiter".to_string(),
            recipient: "recipient".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // only arbiter can refund
        let info = mock_info("creator", &[]);
        let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Refund {}).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // arbiter refunds
        let info = mock_info("arbiter", &[]);
        deps.querier.update_balance(mock_env().contract.address, coins(1000, "earth"));

        let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Refund {}).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "creator".to_string(),
                amount: coins(1000, "earth"),
            })
        );
    }
}
