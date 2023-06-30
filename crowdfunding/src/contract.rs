#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, StdError, CosmosMsg, Empty, Order, BankMsg, coins, Addr, coin};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryResponseWrapper, GetConfigResponse, GetSharesResponse, GetFundersResponse, GetTotalFundsResponse};
use crate::state::{Config,CONFIG,SHARES,TOTAL_SHARES,EXECUTE_MSG};
use crate::rules;

const CONTRACT_NAME: &str = "crates.io:crowdfunding";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        owner: env.contract.address,
        denom: msg.denom,
        goal: msg.goal,
        start: msg.start.unwrap_or(env.block.time),
        deadline: msg.deadline,
        name: msg.name,
        description: msg.description,
    };
    config.validate()?;
    CONFIG.save(deps.storage, &config)?;
    TOTAL_SHARES.save(deps.storage, &Uint128::zero())?;
    EXECUTE_MSG.save(deps.storage, &msg.execute_msg)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("name", config.name))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        Fund {} => {
            rules::HAS_STARTED(&deps, &env, &info)?;
            rules::NOT_CLOSED(&deps, &env, &info)?;
            rules::SENT_FUNDS(&deps, &env, &info)?;
            try_fund(deps, env, info)
        }
        Execute {} => {
            rules::IS_CLOSED(&deps, &env, &info)?;
            rules::FULLY_FUNDED(&deps, &env, &info)?;
            try_execute(deps, env, info)
        }
        Claim {} => {
            rules::IS_CLOSED(&deps, &env, &info)?;
            rules::NOT_FULLY_FUNDED(&deps, &env, &info)?;
            try_claim(deps, env, info)
        }
        Refund {} => {
            rules::IS_CLOSED(&deps, &env, &info)?;
            rules::NOT_FULLY_FUNDED(&deps, &env, &info)?;
            try_refund(deps, env, info)
        }
    }
}


pub fn try_fund(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sent_funds = info
        .funds
        .iter()
        .find_map(|v| {
            if v.denom == config.denom {
                Some(v.amount)
            } else {
                None
            }
        })
        .unwrap_or_else(Uint128::zero);

        SHARES
        .update::<_, StdError>(deps.storage, info.sender, |shares| {
            let mut shares = shares.unwrap_or_default();
            shares += sent_funds;
            Ok(shares)
        })?;
    
        TOTAL_SHARES
        .update::<_, StdError>(deps.storage, |total_shares| {
            let mut total_shares = total_shares;
            total_shares += sent_funds;
            Ok(total_shares)
        })?;
    Ok(Response::new())
}

pub fn try_execute(deps: DepsMut, _env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let execute_msg = EXECUTE_MSG
        .load(deps.storage)?
        .ok_or_else(|| StdError::generic_err("execute_msg not set".to_string()))?;
    // execute can only run once ever.
    EXECUTE_MSG.save(deps.storage, &None)?;
    Ok(Response::new().add_message(execute_msg))
}

pub fn try_refund(deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let contract_balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?
        .amount;
    let total_shares = TOTAL_SHARES.load(deps.storage)?;
    let user_shares = SHARES
        .range(deps.storage, None, None, Order::Ascending)
        // batch execute 30 transfers at a time
        .take(30)
        .collect::<Result<Vec<_>, _>>()?;
    let mut next_shares = total_shares;
    let msgs: Vec<CosmosMsg> = vec![];
    for (addr, shares) in user_shares {
        let refund_amount = contract_balance.multiply_ratio(shares, total_shares);
        let _bank_transfer_msg = CosmosMsg::<Empty>::Bank(BankMsg::Send {
            to_address: addr.to_string(),
            amount: coins(refund_amount.u128(), config.denom.clone()),
        });
        SHARES.remove(deps.storage, addr);
        next_shares -= shares;
    }
    TOTAL_SHARES.save(deps.storage, &next_shares)?;
    Ok(Response::new().add_messages(msgs))
}

pub fn try_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let contract_balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?
        .amount;
    let total_shares = TOTAL_SHARES.load(deps.storage)?;
    let user_shares = SHARES.load(deps.storage, info.sender.clone())?;
    let mut next_total_shares = total_shares;
    let refund_amount = contract_balance.multiply_ratio(user_shares, total_shares);
    let bank_transfer_msg = CosmosMsg::<Empty>::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: coins(refund_amount.u128(), config.denom),
    });
    SHARES.remove(deps.storage, info.sender);
    next_total_shares -= user_shares;
    TOTAL_SHARES.save(deps.storage, &next_total_shares)?;
    Ok(Response::new().add_message(bank_transfer_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let output: StdResult<QueryResponseWrapper> = match msg {
        QueryMsg::GetConfig {} => get_config(deps, env),
        QueryMsg::GetShares { user } => get_shares(deps, env, user),
        QueryMsg::GetFunders { limit, start_after } => {
            get_funders(deps, env, limit, start_after)
        }
        QueryMsg::GetTotalFunds {} => get_funds(deps, env),
    };
    output?.to_binary()
}


pub fn get_config(deps: Deps, _env: Env) -> StdResult<QueryResponseWrapper> {
    let config = CONFIG.load(deps.storage)?;
    Ok(QueryResponseWrapper::GetConfigResponse(GetConfigResponse {
        goal: coin(config.goal.u128(), config.denom),
        deadline: config.deadline,
        name: config.name,
        description: config.description,
    }))
}

pub fn get_shares(deps: Deps, _env: Env, address: String) -> StdResult<QueryResponseWrapper> {
    let addr = deps.api.addr_validate(&address)?;
    let shares = SHARES.load(deps.storage, addr)?;
    Ok(QueryResponseWrapper::GetSharesResponse(GetSharesResponse {
        shares,
        address,
    }))
}

pub fn get_funders(
    deps: Deps,
    _env: Env,
    limit: Uint128,
    start_after: Option<String>,
) -> StdResult<QueryResponseWrapper> {
    let start = start_after
        .map(|s| deps.api.addr_validate(&s))
        .transpose()?
        .map(|addr| Bound::InclusiveRaw::<Addr>(addr.as_bytes().to_vec()));
    let funders = SHARES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit.u128() as usize)
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|(addr, shares)| (addr.to_string(), *shares))
        .collect::<Vec<(String, Uint128)>>();
    Ok(QueryResponseWrapper::GetFundersResponse(
        GetFundersResponse { funders },
    ))
}

pub fn get_funds(deps: Deps, _env: Env) -> StdResult<QueryResponseWrapper> {
    let funds = TOTAL_SHARES.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    Ok(QueryResponseWrapper::GetTotalFundsResponse(
        GetTotalFundsResponse {
            total_funds: coin(funds.u128(), config.denom),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        coins, CosmosMsg, Empty, Uint128,
    };

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let  env = mock_env();
        let info = mock_info("creator", &coins(100, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            denom: "earth".to_string(),
            goal: Uint128::new(100),
            start: None,
            deadline: env.block.time.plus_seconds(86400),
            name: "Crowdfunding Campaign".to_string(),
            description: "Test campaign".to_string(),
            execute_msg: Some(CosmosMsg::Custom(Empty {})),
        };

        let res=instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(res.messages.len(),0);
        
     }

     #[test]
     fn test_fund(){
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("creator", &coins(100, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            denom: "earth".to_string(),
            goal: Uint128::new(100),
            start: None,
            deadline: env.block.time.plus_seconds(86400),
            name: "Crowdfunding Campaign".to_string(),
            description: "Test campaign".to_string(),
            execute_msg: Some(CosmosMsg::Custom(Empty {})),
        };

        let res=instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(res.messages.len(),0);

        env.block.time = env.block.time.plus_seconds(60);

        // Execute with Fund case
        let msg = ExecuteMsg::Fund {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);
     }

     #[test]
     fn test_execute(){
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("creator", &coins(100, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            denom: "earth".to_string(),
            goal: Uint128::new(100),
            start: None,
            deadline: env.block.time.plus_seconds(86400),
            name: "Crowdfunding Campaign".to_string(),
            description: "Test campaign".to_string(),
            execute_msg: Some(CosmosMsg::Custom(Empty {})),
        };

        let res=instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(res.messages.len(),0);

        env.block.time = env.block.time.plus_seconds(60);

        // Execute with Fund case
        let msg = ExecuteMsg::Fund {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // Execute with Execute case
        env.block.time = env.block.time.plus_seconds(86401);
        let msg = ExecuteMsg::Execute {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.messages[0].msg, CosmosMsg::Custom(Empty {}));
     }

     #[test]
     fn test_refund(){
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("creator", &coins(80, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            denom: "earth".to_string(),
            goal: Uint128::new(100),
            start: None,
            deadline: env.block.time.plus_seconds(86400),
            name: "Crowdfunding Campaign".to_string(),
            description: "Test campaign".to_string(),
            execute_msg: Some(CosmosMsg::Custom(Empty {})),
        };

        let res=instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(res.messages.len(),0);

        // Execute with Fund case
        let msg = ExecuteMsg::Fund {};
        env.block.time = env.block.time.plus_seconds(60);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

         // Execute with Refund case
         env.block.time = env.block.time.plus_seconds(86400);
        let msg = ExecuteMsg::Refund {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);
     }

     #[test]
     fn test_claim()
     {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("creator", &coins(80, "earth"));

        // Instantiate the contract
        let msg = InstantiateMsg {
            denom: "earth".to_string(),
            goal: Uint128::new(100),
            start: None,
            deadline: env.block.time.plus_seconds(86400),
            name: "Crowdfunding Campaign".to_string(),
            description: "Test campaign".to_string(),
            execute_msg: Some(CosmosMsg::Custom(Empty {})),
        };

        let res=instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(res.messages.len(),0);

        // Execute with Fund case
        let msg = ExecuteMsg::Fund {};
        env.block.time = env.block.time.plus_seconds(60);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // Execute with Claim case
        env.block.time = env.block.time.plus_seconds(86400);
        let msg = ExecuteMsg::Claim {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 1);
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "creator".to_string(),
                amount: coins(0, "earth"),
            })
        );
     }
}