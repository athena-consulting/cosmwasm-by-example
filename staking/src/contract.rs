#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakedResponse};
use crate::state::{Config, CONFIG, STAKES};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        token_denom: msg.token_denom,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Stake {} => execute_stake(deps, info),
        ExecuteMsg::Unstake { amount } => execute_unstake(deps, info, amount),
    }
}

pub fn execute_stake(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let coin = info
        .funds
        .iter()
        .find(|c| c.denom == config.token_denom)
        .ok_or(ContractError::InsufficientFunds {})?;

    STAKES.update(deps.storage, info.sender.as_str(), |balance| -> StdResult<_> {
        Ok(balance.unwrap_or_default() + coin.amount.u128())
    })?;

    Ok(Response::new().add_attribute("action", "stake"))
}

pub fn execute_unstake(
    deps: DepsMut,
    info: MessageInfo,
    amount: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let amount_u128 = amount
        .parse::<u128>()
        .map_err(|_| ContractError::InvalidAmount {})?;

    let mut current_stake = STAKES.load(deps.storage, info.sender.as_str()).unwrap_or_default();
    if current_stake < amount_u128 {
        return Err(ContractError::InsufficientFunds {});
    }

    current_stake -= amount_u128;
    STAKES.save(deps.storage, info.sender.as_str(), &current_stake)?;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: config.token_denom,
            amount: Uint128::from(amount_u128),
        }],
    };

    Ok(Response::new()
        .add_attribute("action", "unstake")
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Staked { address } => to_binary(&query_staked(deps, address)?),
    }
}

fn query_staked(deps: Deps, address: String) -> StdResult<StakedResponse> {
    let amount = STAKES.load(deps.storage, &address).unwrap_or_default();
    Ok(StakedResponse {
        amount: amount.to_string(),
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
            token_denom: "earth".to_string(),
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_stake() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            token_denom: "earth".to_string(),
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("user", &coins(100, "earth"));
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Stake {}).unwrap();

        let res = query_staked(deps.as_ref(), "user".to_string()).unwrap();
        assert_eq!(res.amount, "100");
    }

    #[test]
    fn reject_invalid_unstake_amount() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            token_denom: "earth".to_string(),
        };
        instantiate(deps.as_mut(), mock_env(), mock_info("creator", &[]), msg).unwrap();

        let info = mock_info("user", &coins(100, "earth"));
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Stake {}).unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("user", &[]),
            ExecuteMsg::Unstake {
                amount: "not-a-number".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::InvalidAmount {});
    }
}
