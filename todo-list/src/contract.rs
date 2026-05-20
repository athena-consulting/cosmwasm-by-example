#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TaskResponse, TasksResponse,
};
use crate::state::{Config, Task, UserStats, CONFIG, TASKS, USER_STATS};

const CONTRACT_NAME: &str = "crates.io:todo-list";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_OPEN_TASKS: u32 = 50;
const MAX_ALLOWED_OPEN_TASKS: u32 = 1000;
const MAX_TITLE_LEN: usize = 120;
const MAX_DESCRIPTION_LEN: usize = 500;
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let max_open_tasks = validate_max_open_tasks(msg.max_open_tasks)?;
    let config = Config {
        owner: info.sender.clone(),
        max_open_tasks,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("max_open_tasks", max_open_tasks.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddTask { title, description } => {
            execute_add_task(deps, env, info, title, description)
        }
        ExecuteMsg::CompleteTask { id } => execute_complete_task(deps, env, info, id),
        ExecuteMsg::ReopenTask { id } => execute_reopen_task(deps, info, id),
        ExecuteMsg::DeleteTask { id } => execute_delete_task(deps, info, id),
        ExecuteMsg::UpdateConfig {
            max_open_tasks,
            new_owner,
        } => execute_update_config(deps, info, max_open_tasks, new_owner),
    }
}

pub fn execute_add_task(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: Option<String>,
) -> Result<Response, ContractError> {
    let title = validate_title(title)?;
    let description = validate_description(description)?;
    let config = CONFIG.load(deps.storage)?;
    let mut stats = USER_STATS
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(UserStats {
            next_id: 1,
            open_count: 0,
        });

    if stats.open_count >= config.max_open_tasks {
        return Err(ContractError::MaxOpenTasksReached {});
    }

    let id = stats.next_id;
    let task = Task {
        title,
        description,
        completed: false,
        created_at_height: env.block.height,
        completed_at_height: None,
    };

    TASKS.save(deps.storage, (&info.sender, id), &task)?;
    stats.next_id += 1;
    stats.open_count += 1;
    USER_STATS.save(deps.storage, &info.sender, &stats)?;

    Ok(Response::new()
        .add_attribute("action", "add_task")
        .add_attribute("owner", info.sender)
        .add_attribute("id", id.to_string())
        .add_attribute("open_count", stats.open_count.to_string()))
}

pub fn execute_complete_task(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut task = load_task(deps.storage, &info.sender, id)?;
    if !task.completed {
        task.completed = true;
        task.completed_at_height = Some(env.block.height);
        decrement_open_count(deps.storage, &info.sender)?;
        TASKS.save(deps.storage, (&info.sender, id), &task)?;
    }

    Ok(Response::new()
        .add_attribute("action", "complete_task")
        .add_attribute("owner", info.sender)
        .add_attribute("id", id.to_string()))
}

pub fn execute_reopen_task(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut task = load_task(deps.storage, &info.sender, id)?;

    if task.completed {
        let mut stats = USER_STATS.load(deps.storage, &info.sender)?;
        if stats.open_count >= config.max_open_tasks {
            return Err(ContractError::MaxOpenTasksReached {});
        }
        task.completed = false;
        task.completed_at_height = None;
        stats.open_count += 1;
        USER_STATS.save(deps.storage, &info.sender, &stats)?;
        TASKS.save(deps.storage, (&info.sender, id), &task)?;
    }

    Ok(Response::new()
        .add_attribute("action", "reopen_task")
        .add_attribute("owner", info.sender)
        .add_attribute("id", id.to_string()))
}

pub fn execute_delete_task(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let task = load_task(deps.storage, &info.sender, id)?;
    TASKS.remove(deps.storage, (&info.sender, id));
    if !task.completed {
        decrement_open_count(deps.storage, &info.sender)?;
    }

    Ok(Response::new()
        .add_attribute("action", "delete_task")
        .add_attribute("owner", info.sender)
        .add_attribute("id", id.to_string()))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    max_open_tasks: Option<u32>,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if max_open_tasks.is_some() {
        config.max_open_tasks = validate_max_open_tasks(max_open_tasks)?;
    }
    if let Some(new_owner) = new_owner {
        config.owner = deps.api.addr_validate(&new_owner)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute("max_open_tasks", config.max_open_tasks.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Task { owner, id } => to_json_binary(&query_task(deps, owner, id)?),
        QueryMsg::Tasks {
            owner,
            start_after,
            limit,
            include_completed,
        } => to_json_binary(&query_tasks(
            deps,
            owner,
            start_after,
            limit,
            include_completed,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.into_string(),
        max_open_tasks: config.max_open_tasks,
    })
}

pub fn query_task(deps: Deps, owner: String, id: u64) -> StdResult<TaskResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let task = TASKS.load(deps.storage, (&owner_addr, id))?;
    Ok(task_response(owner_addr, id, task))
}

pub fn query_tasks(
    deps: Deps,
    owner: String,
    start_after: Option<u64>,
    limit: Option<u32>,
    include_completed: Option<bool>,
) -> StdResult<TasksResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let include_completed = include_completed.unwrap_or(true);

    let tasks = TASKS
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|item| {
            include_completed
                || item
                    .as_ref()
                    .map(|(_, task)| !task.completed)
                    .unwrap_or(true)
        })
        .take(limit)
        .map(|item| {
            let (id, task) = item?;
            Ok(task_response(owner_addr.clone(), id, task))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(TasksResponse { tasks })
}

fn load_task(
    storage: &dyn cosmwasm_std::Storage,
    owner: &cosmwasm_std::Addr,
    id: u64,
) -> Result<Task, ContractError> {
    TASKS
        .may_load(storage, (owner, id))?
        .ok_or(ContractError::TaskNotFound {})
}

fn decrement_open_count(
    storage: &mut dyn cosmwasm_std::Storage,
    owner: &cosmwasm_std::Addr,
) -> Result<(), ContractError> {
    USER_STATS.update(storage, owner, |stats| -> Result<_, ContractError> {
        let mut stats = stats.ok_or(ContractError::TaskNotFound {})?;
        stats.open_count = stats.open_count.saturating_sub(1);
        Ok(stats)
    })?;
    Ok(())
}

fn task_response(owner: cosmwasm_std::Addr, id: u64, task: Task) -> TaskResponse {
    TaskResponse {
        id,
        owner: owner.into_string(),
        title: task.title,
        description: task.description,
        completed: task.completed,
        created_at_height: task.created_at_height,
        completed_at_height: task.completed_at_height,
    }
}

fn validate_title(title: String) -> Result<String, ContractError> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ContractError::EmptyTitle {});
    }
    if title.chars().count() > MAX_TITLE_LEN {
        return Err(ContractError::TitleTooLong {});
    }
    Ok(title)
}

fn validate_description(description: Option<String>) -> Result<Option<String>, ContractError> {
    description
        .map(|value| {
            let value = value.trim().to_string();
            if value.chars().count() > MAX_DESCRIPTION_LEN {
                Err(ContractError::DescriptionTooLong {})
            } else if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        })
        .transpose()
        .map(Option::flatten)
}

fn validate_max_open_tasks(max_open_tasks: Option<u32>) -> Result<u32, ContractError> {
    let max_open_tasks = max_open_tasks.unwrap_or(DEFAULT_MAX_OPEN_TASKS);
    if max_open_tasks == 0 || max_open_tasks > MAX_ALLOWED_OPEN_TASKS {
        return Err(ContractError::InvalidMaxOpenTasks {});
    }
    Ok(max_open_tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{from_json, Env, MemoryStorage, OwnedDeps};

    fn setup() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            InstantiateMsg {
                max_open_tasks: Some(2),
            },
        )
        .unwrap();
        (deps, env)
    }

    fn add_task(deps: DepsMut, env: Env, owner: &str, title: &str) {
        execute(
            deps,
            env,
            mock_info(owner, &[]),
            ExecuteMsg::AddTask {
                title: title.to_string(),
                description: Some("details".to_string()),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_sets_config() {
        let (deps, env) = setup();
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();

        assert_eq!(config.owner, "admin");
        assert_eq!(config.max_open_tasks, 2);
    }

    #[test]
    fn user_can_add_and_query_task() {
        let (mut deps, mut env) = setup();
        env.block.height = 42;

        add_task(deps.as_mut(), env.clone(), "alice", " Write tests ");

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Task {
                owner: "alice".to_string(),
                id: 1,
            },
        )
        .unwrap();
        let task: TaskResponse = from_json(&res).unwrap();

        assert_eq!(task.id, 1);
        assert_eq!(task.owner, "alice");
        assert_eq!(task.title, "Write tests");
        assert_eq!(task.description, Some("details".to_string()));
        assert!(!task.completed);
        assert_eq!(task.created_at_height, 42);
    }

    #[test]
    fn max_open_tasks_is_enforced() {
        let (mut deps, env) = setup();

        add_task(deps.as_mut(), env.clone(), "alice", "one");
        add_task(deps.as_mut(), env.clone(), "alice", "two");

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::AddTask {
                title: "three".to_string(),
                description: None,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::MaxOpenTasksReached {});
    }

    #[test]
    fn complete_reopen_and_delete_update_open_count() {
        let (mut deps, env) = setup();
        add_task(deps.as_mut(), env.clone(), "alice", "one");
        add_task(deps.as_mut(), env.clone(), "alice", "two");

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::CompleteTask { id: 1 },
        )
        .unwrap();

        add_task(deps.as_mut(), env.clone(), "alice", "three");

        let reopen_err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::ReopenTask { id: 1 },
        )
        .unwrap_err();
        assert_eq!(reopen_err, ContractError::MaxOpenTasksReached {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::DeleteTask { id: 2 },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::ReopenTask { id: 1 },
        )
        .unwrap();
    }

    #[test]
    fn tasks_query_filters_completed_and_paginates() {
        let (mut deps, env) = setup();
        add_task(deps.as_mut(), env.clone(), "alice", "one");
        add_task(deps.as_mut(), env.clone(), "alice", "two");

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::CompleteTask { id: 1 },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Tasks {
                owner: "alice".to_string(),
                start_after: Some(1),
                limit: Some(10),
                include_completed: Some(false),
            },
        )
        .unwrap();
        let tasks: TasksResponse = from_json(&res).unwrap();

        assert_eq!(tasks.tasks.len(), 1);
        assert_eq!(tasks.tasks[0].id, 2);
    }

    #[test]
    fn only_owner_can_update_config() {
        let (mut deps, env) = setup();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateConfig {
                max_open_tasks: Some(10),
                new_owner: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                max_open_tasks: Some(10),
                new_owner: Some("alice".to_string()),
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.owner, "alice");
        assert_eq!(config.max_open_tasks, 10);
    }
}
