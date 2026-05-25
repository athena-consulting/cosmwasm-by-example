#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WinnerResponse};
use crate::state::{Config, CONFIG, TICKETS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        ticket_price: msg
            .ticket_price
            .parse()
            .map_err(|_| ContractError::InvalidTicketPrice {})?,
        ticket_denom: msg.ticket_denom,
        winner: "".to_string(),
    };
    CONFIG.save(deps.storage, &config)?;
    TICKETS.save(deps.storage, &vec![])?;
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
        ExecuteMsg::BuyTicket {} => execute_buy_ticket(deps, info),
        ExecuteMsg::DrawWinner {} => execute_draw(deps, env),
    }
}

pub fn execute_buy_ticket(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.winner.is_empty() {
        return Err(ContractError::WinnerAlreadyDrawn {});
    }

    let coin = info
        .funds
        .iter()
        .find(|c| c.denom == config.ticket_denom)
        .ok_or(ContractError::InsufficientFunds {})?;

    if coin.amount.u128() < config.ticket_price {
        return Err(ContractError::InsufficientFunds {});
    }

    let mut tickets = TICKETS.load(deps.storage)?;
    tickets.push(info.sender.to_string());
    TICKETS.save(deps.storage, &tickets)?;

    Ok(Response::new().add_attribute("action", "buy_ticket"))
}

pub fn execute_draw(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if !config.winner.is_empty() {
        return Err(ContractError::WinnerAlreadyDrawn {});
    }

    let tickets = TICKETS.load(deps.storage)?;
    if tickets.is_empty() {
        return Err(ContractError::NoPlayers {});
    }

    // Pseudo-random selection based on block time
    let idx = (env.block.time.seconds() as usize) % tickets.len();
    let winner = tickets[idx].clone();

    config.winner = winner.clone();
    CONFIG.save(deps.storage, &config)?;

    let balance = deps.querier.query_all_balances(&env.contract.address)?;

    let msg = BankMsg::Send {
        to_address: winner,
        amount: balance,
    };

    Ok(Response::new()
        .add_attribute("action", "draw_winner")
        .add_message(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Winner {} => to_binary(&query_winner(deps)?),
    }
}

fn query_winner(deps: Deps) -> StdResult<WinnerResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(WinnerResponse {
        winner: config.winner,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            ticket_price: "100".to_string(),
            ticket_denom: "earth".to_string(),
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn reject_invalid_ticket_price() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            ticket_price: "not-a-number".to_string(),
            ticket_denom: "earth".to_string(),
        };
        let info = mock_info("creator", &[]);

        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();

        assert_eq!(err, ContractError::InvalidTicketPrice {});
    }

    #[test]
    fn test_buy_and_draw() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            ticket_price: "100".to_string(),
            ticket_denom: "earth".to_string(),
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("user", &coins(100, "earth"));
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::BuyTicket {}).unwrap();

        // draw
        let mut env = mock_env();
        env.block.time = cosmwasm_std::Timestamp::from_seconds(0);
        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), env, info, ExecuteMsg::DrawWinner {}).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
