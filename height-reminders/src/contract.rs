#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, ListRemindersResponse, QueryMsg, ReminderResponse,
};
use crate::state::{Config, Reminder, CONFIG, REMINDERS};

const CONTRACT_NAME: &str = "crates.io:height-reminders";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateReminder {
            id,
            due_height,
            note,
        } => execute_create_reminder(deps, info, id, due_height, note),
        ExecuteMsg::CompleteReminder { id } => execute_complete_reminder(deps, env, id),
        ExecuteMsg::CancelReminder { id } => execute_cancel_reminder(deps, info, id),
    }
}

fn execute_create_reminder(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
    due_height: u64,
    note: String,
) -> Result<Response, ContractError> {
    let id = id.trim().to_string();
    let note = note.trim().to_string();
    if id.is_empty() || note.is_empty() {
        return Err(ContractError::EmptyField {});
    }
    if REMINDERS.has(deps.storage, &id) {
        return Err(ContractError::ReminderExists {});
    }

    let reminder = Reminder {
        id: id.clone(),
        creator: info.sender,
        due_height,
        note,
        completed: false,
    };
    REMINDERS.save(deps.storage, &id, &reminder)?;

    Ok(Response::new()
        .add_attribute("action", "create_reminder")
        .add_attribute("id", id)
        .add_attribute("due_height", due_height.to_string()))
}

fn execute_complete_reminder(
    deps: DepsMut,
    env: Env,
    id: String,
) -> Result<Response, ContractError> {
    REMINDERS.update(deps.storage, &id, |record| -> Result<_, ContractError> {
        let mut reminder = record.ok_or(ContractError::ReminderNotFound {})?;
        if env.block.height < reminder.due_height {
            return Err(ContractError::NotDue {});
        }
        reminder.completed = true;
        Ok(reminder)
    })?;

    Ok(Response::new()
        .add_attribute("action", "complete_reminder")
        .add_attribute("id", id))
}

fn execute_cancel_reminder(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
) -> Result<Response, ContractError> {
    let reminder = REMINDERS
        .load(deps.storage, &id)
        .map_err(|_| ContractError::ReminderNotFound {})?;
    if reminder.creator != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    REMINDERS.remove(deps.storage, &id);

    Ok(Response::new()
        .add_attribute("action", "cancel_reminder")
        .add_attribute("id", id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Reminder { id } => to_json_binary(&query_reminder(deps, id)?),
        QueryMsg::ListReminders {} => to_json_binary(&query_reminders(deps)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_reminder(deps: Deps, id: String) -> StdResult<ReminderResponse> {
    let reminder = REMINDERS.load(deps.storage, &id)?;
    Ok(ReminderResponse { reminder })
}

fn query_reminders(deps: Deps) -> StdResult<ListRemindersResponse> {
    let ids = REMINDERS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<String>>>()?;
    Ok(ListRemindersResponse { ids })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    fn init() -> cosmwasm_std::OwnedDeps<
        cosmwasm_std::MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    > {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg {},
        )
        .unwrap();
        deps
    }

    #[test]
    fn creates_and_queries_reminder() {
        let mut deps = init();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateReminder {
                id: "renewal".to_string(),
                due_height: 100,
                note: "Renew validator delegation".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Reminder {
                id: "renewal".to_string(),
            },
        )
        .unwrap();
        let value: ReminderResponse = from_json(&res).unwrap();
        assert_eq!(
            value.reminder.creator,
            cosmwasm_std::Addr::unchecked("alice")
        );
        assert_eq!(value.reminder.due_height, 100);
        assert!(!value.reminder.completed);
    }

    #[test]
    fn blocks_duplicate_and_empty_reminders() {
        let mut deps = init();
        let msg = ExecuteMsg::CreateReminder {
            id: "renewal".to_string(),
            due_height: 100,
            note: "Renew validator delegation".to_string(),
        };

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            msg.clone(),
        )
        .unwrap();
        let err = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap_err();
        assert_eq!(err, ContractError::ReminderExists {});

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateReminder {
                id: " ".to_string(),
                due_height: 100,
                note: "note".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::EmptyField {});
    }

    #[test]
    fn completes_only_after_due_height() {
        let mut deps = init();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateReminder {
                id: "go-live".to_string(),
                due_height: 25,
                note: "Launch checklist".to_string(),
            },
        )
        .unwrap();

        let mut env = mock_env();
        env.block.height = 24;
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("bob", &[]),
            ExecuteMsg::CompleteReminder {
                id: "go-live".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NotDue {});

        let mut env = mock_env();
        env.block.height = 25;
        execute(
            deps.as_mut(),
            env,
            mock_info("bob", &[]),
            ExecuteMsg::CompleteReminder {
                id: "go-live".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Reminder {
                id: "go-live".to_string(),
            },
        )
        .unwrap();
        let value: ReminderResponse = from_json(&res).unwrap();
        assert!(value.reminder.completed);
    }

    #[test]
    fn only_creator_can_cancel() {
        let mut deps = init();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateReminder {
                id: "tax".to_string(),
                due_height: 30,
                note: "Prepare report".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::CancelReminder {
                id: "tax".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CancelReminder {
                id: "tax".to_string(),
            },
        )
        .unwrap();

        let err = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Reminder {
                id: "tax".to_string(),
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn lists_reminder_ids() {
        let mut deps = init();
        for id in ["b", "a"] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info("alice", &[]),
                ExecuteMsg::CreateReminder {
                    id: id.to_string(),
                    due_height: 10,
                    note: format!("note {id}"),
                },
            )
            .unwrap();
        }

        let res = query(deps.as_ref(), mock_env(), QueryMsg::ListReminders {}).unwrap();
        let value: ListRemindersResponse = from_json(&res).unwrap();
        assert_eq!(value.ids, vec!["a".to_string(), "b".to_string()]);
    }
}
