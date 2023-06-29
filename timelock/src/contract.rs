#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::WasmMsg::Execute;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError,
    StdResult, Uint64,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{Duration, Scheduled};
use std::ops::Add;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, OperationListResponse, QueryMsg};
use crate::state::{Operation, OperationStatus, Timelock, CONFIG, OPERATION_LIST, OPERATION_SEQ};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw3-timelock";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut admins = vec![];
    match msg.admins {
        None => {
            admins.push(info.sender.clone());
        }
        Some(admin_list) => {
            for admin in admin_list {
                admins.push(deps.api.addr_validate(&admin)?);
            }
        }
    }
    admins.push(env.contract.address);

    let mut proposers = vec![];
    for proposer in msg.proposers {
        proposers.push(deps.api.addr_validate(&proposer)?);
    }

    let timelock = Timelock {
        min_time_delay: msg.min_delay,
        proposers,
        admins,
        frozen: false,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OPERATION_SEQ.save(deps.storage, &Uint64::zero())?;
    CONFIG.save(deps.storage, &timelock)?;

    Ok(Response::new()
        .add_attribute("Method: ", "instantiate")
        .add_attribute("Admin: ", info.sender)
        .add_attribute(
            "Proposers: ",
            timelock
                .proposers
                .into_iter()
                .map(|item| item.to_string())
                .collect::<String>(),
        )
        .add_attribute("minTimeDelay: ", timelock.min_time_delay.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Schedule {
            target_address,
            data,
            title,
            description,
            execution_time,
            executors,
        } => execute_schedule(
            deps,
            _env,
            info,
            target_address,
            data,
            title,
            description,
            execution_time,
            executors,
        ),
        ExecuteMsg::Execute { operation_id } => execute_execute(deps, _env, info, operation_id),
        ExecuteMsg::Cancel { operation_id } => execute_cancel(deps, _env, info, operation_id),
        ExecuteMsg::RevokeAdmin { admin_address } => {
            execute_revoke_admin(deps, _env, info, admin_address)
        }
        ExecuteMsg::AddProposer { proposer_address } => {
            execute_add_proposer(deps, _env, info, proposer_address)
        }
        ExecuteMsg::RemoveProposer { proposer_address } => {
            execute_remove_proposer(deps, _env, info, proposer_address)
        }
        ExecuteMsg::UpdateMinDelay { new_delay } => {
            execute_update_min_delay(deps, _env, info, new_delay)
        }
        ExecuteMsg::Freeze {} => execute_freeze(deps, _env, info),
    }
}

/*eslint too-many-arguments-threshold:9 */
#[allow(clippy::too_many_arguments)]
pub fn execute_schedule(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target_address: String,
    data: Binary,
    title: String,
    description: String,
    execution_time: Scheduled,
    executor_list: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_validate(&info.sender.to_string())?;
    let target = deps.api.addr_validate(&target_address)?;

    let timelock = CONFIG.load(deps.storage)?;
    if !(timelock.proposers.contains(&sender)) {
        return Err(ContractError::Unauthorized {});
    }

    if Scheduled::AtTime(env.block.time).add(timelock.min_time_delay)? > execution_time {
        return Err(ContractError::MinDelayNotSatisfied {});
    }

    let id = OPERATION_SEQ.update::<_, StdError>(deps.storage, |id| Ok(id.add(Uint64::new(1))))?;

    let mut executors = None;
    match executor_list {
        None => {}
        Some(list) => {
            let mut checked_executors = vec![];
            for executor in list {
                checked_executors.push(deps.api.addr_validate(&executor)?);
            }
            executors = Option::from(checked_executors);
        }
    }

    let new_operation = Operation {
        id,
        status: OperationStatus::Pending,
        proposer: sender,
        executors,
        execution_time,
        target,
        data,
        title,
        description,
    };
    OPERATION_LIST.save(deps.storage, id.u64(), &new_operation)?;

    Ok(Response::new()
        .add_attribute("Schedule ", "success")
        .add_attribute("Operation ID: ", id)
        .add_attribute("Proposer: ", new_operation.proposer)
        .add_attribute("Target Address: ", new_operation.target.to_string())
        .add_attribute("Execution Time: ", new_operation.execution_time.to_string()))
}

pub fn execute_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operation_id: Uint64,
) -> Result<Response, ContractError> {
    let mut operation = OPERATION_LIST.load(deps.storage, operation_id.u64())?;

    //is delay ended
    if !operation.execution_time.is_triggered(&env.block) {
        return Err(ContractError::Unexpired {});
    }
    //has executer list if so sender is in it
    if operation.executors.is_some()
        && !operation
            .executors
            .clone()
            .map(|c| c.contains(&info.sender))
            .unwrap()
    {
        return Err(ContractError::Unauthorized {});
    }

    if operation.status == OperationStatus::Done {
        return Err(ContractError::Executed {});
    }

    //change operation status
    operation.status = OperationStatus::Done;
    OPERATION_LIST.save(deps.storage, operation_id.u64(), &operation)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(Execute {
            contract_addr: operation.target.to_string(),
            msg: operation.data,
            funds: vec![],
        }))
        .add_attribute("executor", &info.sender.to_string()))
}

pub fn execute_cancel(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operation_id: Uint64,
) -> Result<Response, ContractError> {
    let operation = OPERATION_LIST.load(deps.storage, operation_id.u64())?;

    if operation.status == OperationStatus::Done {
        return Err(ContractError::NotDeletable {});
    }

    if operation.proposer != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    OPERATION_LIST.remove(deps.storage, operation_id.u64());

    Ok(Response::new()
        .add_attribute("Method", "cancel")
        .add_attribute("sender", &info.sender.to_string())
        .add_attribute("operation_id", operation_id.to_string())
        .add_attribute("Result", "Success"))
}

pub fn execute_revoke_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin_address: String,
) -> Result<Response, ContractError> {
    let mut timelock = CONFIG.load(deps.storage)?;
    if timelock.frozen {
        return Err(ContractError::TimelockFrozen {});
    }
    if !timelock.admins.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let admin_address = deps.api.addr_validate(&admin_address)?;

    let index = timelock
        .admins
        .iter()
        .position(|x| *x == admin_address.clone())
        .ok_or(ContractError::NotFound {
            address: admin_address.clone().to_string(),
        })?;

    timelock.admins.remove(index);
    CONFIG.save(deps.storage, &timelock)?;
    Ok(Response::new()
        .add_attribute("Method", "revoke admin")
        .add_attribute("sender", &info.sender)
        .add_attribute("Admin to revoke", admin_address)
        .add_attribute("Result", "Success"))
}

pub fn execute_add_proposer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proposer_address: String,
) -> Result<Response, ContractError> {
    let mut timelock = CONFIG.load(deps.storage)?;

    if timelock.frozen {
        return Err(ContractError::TimelockFrozen {});
    }

    if !timelock.admins.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let proposer_address = deps.api.addr_validate(&proposer_address)?;

    //is in proposers list
    if timelock.proposers.contains(&proposer_address) {
        return Err(ContractError::AlreadyContainsProposerAddress {});
    }

    timelock.proposers.push(proposer_address);
    CONFIG.save(deps.storage, &timelock)?;
    Ok(Response::new()
        .add_attribute("Method", "add_proposer")
        .add_attribute("sender", &info.sender)
        .add_attribute("Result", "Success"))
}

pub fn execute_remove_proposer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    proposer_address: String,
) -> Result<Response, ContractError> {
    let mut timelock = CONFIG.load(deps.storage)?;

    if timelock.frozen {
        return Err(ContractError::TimelockFrozen {});
    }

    if !timelock.admins.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let proposer_address = deps.api.addr_validate(&proposer_address)?;
    //is in proposers
    let index = timelock
        .proposers
        .iter()
        .position(|x| *x == proposer_address.clone())
        .ok_or(ContractError::NotFound {
            address: proposer_address.clone().to_string(),
        })?;

    timelock.proposers.remove(index);
    CONFIG.save(deps.storage, &timelock)?;
    Ok(Response::new()
        .add_attribute("Method", "remove_proposer")
        .add_attribute("sender", &info.sender)
        .add_attribute("Result", "Success"))
}

pub fn execute_update_min_delay(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_delay: Duration,
) -> Result<Response, ContractError> {
    let mut timelock = CONFIG.load(deps.storage)?;

    if timelock.frozen {
        return Err(ContractError::TimelockFrozen {});
    }

    if !timelock.admins.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    timelock.min_time_delay = new_delay;

    CONFIG.save(deps.storage, &timelock)?;
    Ok(Response::new()
        .add_attribute("Method", "Update Min Delay")
        .add_attribute("Sender", &info.sender.to_string())
        .add_attribute("New Min Delay", timelock.min_time_delay.to_string())
        .add_attribute("Result", "Success"))
}
pub fn execute_freeze(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut timelock = CONFIG.load(deps.storage)?;

    if timelock.frozen {
        return Err(ContractError::TimelockFrozen {});
    }

    if !timelock.admins.contains(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    timelock.frozen = true;

    CONFIG.save(deps.storage, &timelock)?;

    Ok(Response::new()
        .add_attribute("Method", "freeze")
        .add_attribute("sender", &info.sender)
        .add_attribute("Result", "Success"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOperationStatus { operation_id } => {
            to_binary(&query_get_operation_status(deps, operation_id)?)
        }
        QueryMsg::GetExecutionTime { operation_id } => {
            to_binary(&query_get_execution_time(deps, operation_id)?)
        }
        QueryMsg::GetAdmins {} => to_binary(&query_get_admins(deps)?),
        QueryMsg::GetOperations { start_after, limit } => {
            to_binary(&query_get_operations(deps, start_after, limit)?)
        }
        QueryMsg::GetMinDelay {} => to_binary(&query_get_min_delay(deps)?),
        QueryMsg::GetProposers {} => to_binary(&query_get_proposers(deps)?),
        QueryMsg::GetExecutors { operation_id } => {
            to_binary(&query_get_executors(deps, operation_id)?)
        }
    }
}

pub fn query_get_operation_status(deps: Deps, operation_id: Uint64) -> StdResult<OperationStatus> {
    let operation = OPERATION_LIST.load(deps.storage, operation_id.u64())?;
    Ok(operation.status)
}

pub fn query_get_execution_time(deps: Deps, operation_id: Uint64) -> StdResult<String> {
    let operation = OPERATION_LIST.load(deps.storage, operation_id.u64())?;
    Ok(operation.execution_time.to_string())
}

pub fn query_get_admins(deps: Deps) -> StdResult<Vec<Addr>> {
    let timelock = CONFIG.load(deps.storage)?;
    Ok(timelock.admins)
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_get_operations(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<OperationListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let operations: StdResult<Vec<_>> = OPERATION_LIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let res = OperationListResponse {
        operationList: operations?.into_iter().map(|l| l.1.into()).collect(),
    };
    Ok(res)
}

pub fn query_get_min_delay(deps: Deps) -> StdResult<String> {
    let timelock = CONFIG.load(deps.storage)?;
    Ok(timelock.min_time_delay.to_string())
}

pub fn query_get_proposers(deps: Deps) -> StdResult<Vec<Addr>> {
    let timelock = CONFIG.load(deps.storage)?;
    Ok(timelock.proposers)
}

pub fn query_get_executors(deps: Deps, operation_id: Uint64) -> StdResult<Vec<Addr>> {
    let operation = OPERATION_LIST.load(deps.storage, operation_id.u64())?;
    Ok(operation.executors.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Timestamp;
    use cw_utils::Scheduled;

    #[test]
    fn test_no_executers() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = InstantiateMsg {
            admins: Option::Some(vec!["owner".to_string(), "new_one".to_string()]),
            proposers: vec!["prop1".to_string(), "prop2".to_string()],
            min_delay: Duration::Time(10),
        };
        let info = mock_info("creator", &[]);
        let description = "test desc".to_string();
        let title = "Title Example ".to_string();
        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        println!("{:?}", res);

        let data = to_binary(&"data").unwrap();
        // try Schedule() with sender "creator"
        let res = execute_schedule(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "target".to_string(),
            data.clone(),
            title.clone(),
            description.clone(),
            Scheduled::AtTime(Timestamp::from_seconds(10)),
            Option::None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        //change sender to prop1
        let info = mock_info("prop1", &[]);
        //try Schedule() sender "prop1" execution_time < env.block.time
        let res = execute_schedule(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "target".to_string(),
            data.clone(),
            title.clone(),
            description.clone(),
            Scheduled::AtTime(Timestamp::from_seconds(1)),
            Option::None,
        )
        .unwrap_err();
        assert_eq!(res, ContractError::MinDelayNotSatisfied {});

        //Schedule() sender "prop1" execution_time > env.block.time && min_delay_time > execution_time - env.block.time
        let res = execute_schedule(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "target".to_string(),
            data.clone(),
            title.clone(),
            description.clone(),
            Scheduled::AtTime(Timestamp::from_seconds(120)),
            Option::None,
        )
        .unwrap();
        println!("{:?}", res);

        let res = query_get_execution_time(deps.as_ref(), Uint64::new(1));
        println!("{:?}, {}", res, env.block.time);

        //try Execute() sender "prop1" execution_time > env.block.time
        let res =
            execute_execute(deps.as_mut(), env.clone(), info.clone(), Uint64::new(1)).unwrap_err();
        assert_eq!(res, ContractError::Unexpired {});

        //time pass
        env.block.time = Timestamp::from_seconds(120);
        //try Execute() sender "prop1" execution_time <= env.block.time executors "none"
        let res =
            execute_execute(deps.as_mut(), env.clone(), info.clone(), Uint64::new(1)).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_with_executors() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = InstantiateMsg {
            admins: Option::Some(vec!["owner".to_string(), "newone".to_string()]),
            proposers: vec!["prop1".to_string(), "prop2".to_string()],
            min_delay: Duration::Time(10),
        };
        let info = mock_info("creator", &[]);
        let title = "Title Example ".to_string();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        println!("{:?}", res);

        let data = to_binary(&"data").unwrap();
        let description = "test desc".to_string();
        //change sender to prop1
        let info = mock_info("prop1", &[]);

        //Schedule() sender "prop1" execution_time > env.block.time && min_delay_time > execution_time - env.block.time
        let res = execute_schedule(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "target".to_string(),
            data.clone(),
            title.clone(),
            description.clone(),
            Scheduled::AtTime(Timestamp::from_seconds(120)),
            Option::Some(vec!["exec1".to_string(), "exec2".to_string()]),
        )
        .unwrap();
        println!("{:?}", res);

        let res =
            query_get_operations(deps.as_ref(), Option::Some(0u64), Option::Some(1u32)).unwrap();
        println!("{:?}", res);
        //time pass
        env.block.time = Timestamp::from_seconds(120);

        //try Execute() sender "prop1" execution_time <= env.block.time executors "exec1, exec2"
        let res =
            execute_execute(deps.as_mut(), env.clone(), info.clone(), Uint64::new(1)).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        let info = mock_info("exec1", &[]);
        //Execute() sender "exec1" execution_time <= env.block.time executors "exec1, exec2"
        let res =
            execute_execute(deps.as_mut(), env.clone(), info.clone(), Uint64::new(1)).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_cancel() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = InstantiateMsg {
            admins: Option::Some(vec!["owner".to_string(), "newone".to_string()]),
            proposers: vec!["prop1".to_string(), "prop2".to_string()],
            min_delay: Duration::Time(10),
        };
        let info = mock_info("creator", &[]);
        let title = "Title Example ".to_string();

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        println!("{:?}", res);

        let data = to_binary(&"data").unwrap();
        let description = "test desc".to_string();

        //change sender to prop1
        let info = mock_info("prop1", &[]);

        //Schedule() sender "prop1" execution_time > env.block.time && min_delay_time > execution_time - env.block.time
        let res = execute_schedule(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "target".to_string(),
            data.clone(),
            title.clone(),
            description.clone(),
            Scheduled::AtTime(Timestamp::from_seconds(120)),
            Option::None,
        )
        .unwrap();
        println!("{:?}", res);

        //time pass
        env.block.time = Timestamp::from_seconds(120);

        //Execute() sender "prop1" executors ""
        let res =
            execute_execute(deps.as_mut(), env.clone(), info.clone(), Uint64::new(1)).unwrap();
        println!("{:?}", res);

        //try Cancel() sender "prop1" operation_id "1" status "OperationStatus::Done"
        let res =
            execute_cancel(deps.as_mut(), env.clone(), info.clone(), Uint64::new(1)).unwrap_err();
        assert_eq!(res, ContractError::NotDeletable {});

        //Schedule() sender "prop1"
        let res = execute_schedule(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "target".to_string(),
            data.clone(),
            title.clone(),
            description.clone(),
            Scheduled::AtTime(Timestamp::from_seconds(140)),
            Option::None,
        )
        .unwrap();
        println!("{:?}", res);

        //Cancel() sender "prop1" operation_id "2" status "OperationStatus::Pending"
        let res = execute_cancel(deps.as_mut(), env.clone(), info.clone(), Uint64::new(2)).unwrap();
        println!("{:?}", res);

        //try Cancel() sender "nobody" operation_id "2" admin "creator" proposers "prop1, prop2"
        let res =
            execute_cancel(deps.as_mut(), env.clone(), info.clone(), Uint64::new(2)).unwrap_err();
        println!("{:?}", res);

        //Schedule() sender "prop1"
        let res = execute_schedule(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "target".to_string(),
            data.clone(),
            title.clone(),
            description.clone(),
            Scheduled::AtTime(Timestamp::from_seconds(140)),
            Option::None,
        )
        .unwrap();
        println!("{:?}", res);

        let info = mock_info("nobody", &[]);
        //try Cancel() sender "nobody" operation_id "3" admin "creator" proposers "prop1, prop2"
        let res =
            execute_cancel(deps.as_mut(), env.clone(), info.clone(), Uint64::new(3)).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }

    #[test]
    fn test_add_remove_proposer() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = InstantiateMsg {
            admins: Option::None,
            proposers: vec![],
            min_delay: Duration::Time(10),
        };
        let info = mock_info("creator", &[]);

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        println!("{:?}", res);

        //try remove_proposer sender "creator" proposer_address "prop1" proposers ""
        let res = execute_remove_proposer(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "prop1".to_string(),
        )
        .unwrap_err();
        assert_eq!(
            res,
            ContractError::NotFound {
                address: "prop1".to_string()
            }
        );

        let info = mock_info("no_admin", &[]);
        //try remove_proposer sender "no_admin" proposer_address "prop1" proposers ""
        let res = execute_remove_proposer(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "prop1".to_string(),
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        //try add_proposer sender "no_admin" proposer_address "prop1" proposers ""
        let res = execute_add_proposer(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "prop1".to_string(),
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        let info = mock_info("creator", &[]);
        //add_proposer sender "creator" proposer_address "prop1" proposers ""
        let res = execute_add_proposer(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "prop1".to_string(),
        )
        .unwrap();
        println!("{:?}", res);

        //remove_proposer sender "no_admin" proposer_address "prop1" proposers "prop1"
        let res = execute_remove_proposer(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "prop1".to_string(),
        )
        .unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_update_min_delay() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = InstantiateMsg {
            admins: Option::None,
            proposers: vec![],
            min_delay: Duration::Time(10),
        };
        let info = mock_info("creator", &[]);

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        println!("{:?}", res);

        //update_min_delay() sender "creator"
        let res = execute_update_min_delay(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Duration::Time(100),
        )
        .unwrap();
        println!("{:?}", res);

        let info = mock_info("no_admin", &[]);
        //try update_min_delay() sender "no_admin"
        let res = execute_update_min_delay(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            Duration::Time(100),
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }

    #[test]
    fn test_revoke_admin() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let msg = InstantiateMsg {
            admins: Option::None,
            proposers: vec![],
            min_delay: Duration::Time(10),
        };
        let info = mock_info("creator", &[]);

        // instantiate
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        println!("{:?}", res);

        //try revoke_admin() sender "creator" admin_address "not_in_it" admin "creator"
        let res = execute_revoke_admin(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "not_in_it".to_string(),
        )
        .unwrap_err();
        assert_eq!(
            res,
            ContractError::NotFound {
                address: "not_in_it".to_string()
            }
        );

        //revoke_admin() sender "creator" admin_address "creator" admin "creator"
        let res = execute_revoke_admin(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "creator".to_string(),
        )
        .unwrap();
        println!("{:?}", res);

        //try revoke_admin() sender "creator" admin_address "creator" admin ""
        let res = execute_revoke_admin(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            "creator".to_string(),
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }
}
