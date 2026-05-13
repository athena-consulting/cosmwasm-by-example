#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, ProjectListResponse, ProjectResponse, QueryMsg,
};
use crate::state::{Config, Project, CONFIG, PROJECTS};

const CONTRACT_NAME: &str = "crates.io:project-status-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender.clone(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &Config { owner: owner.clone() })?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddProject {
            id,
            title,
            status,
            note,
        } => execute_add_project(deps, env, info, id, title, status, note),
        ExecuteMsg::UpdateStatus { id, status, note } => {
            execute_update_status(deps, env, info, id, status, note)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            execute_transfer_ownership(deps, info, new_owner)
        }
    }
}

pub fn execute_add_project(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    title: String,
    status: crate::msg::ProjectStatus,
    note: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info)?;

    if PROJECTS.has(deps.storage, &id) {
        return Err(ContractError::ProjectAlreadyExists {});
    }

    let project = Project {
        id: id.clone(),
        title,
        status,
        note,
        updated_by: info.sender.clone(),
        updated_at: env.block.time.seconds(),
    };

    PROJECTS.save(deps.storage, &id, &project)?;

    Ok(Response::new()
        .add_attribute("action", "add_project")
        .add_attribute("project_id", id)
        .add_attribute("updated_by", info.sender))
}

pub fn execute_update_status(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    status: crate::msg::ProjectStatus,
    note: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info)?;

    let project = PROJECTS
        .update(deps.storage, &id, |project| match project {
            Some(mut project) => {
                project.status = status;
                project.note = note;
                project.updated_by = info.sender.clone();
                project.updated_at = env.block.time.seconds();
                Ok(project)
            }
            None => Err(ContractError::ProjectNotFound {}),
        })?;

    Ok(Response::new()
        .add_attribute("action", "update_status")
        .add_attribute("project_id", project.id)
        .add_attribute("updated_by", info.sender))
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info)?;
    let new_owner = deps.api.addr_validate(&new_owner)?;

    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        config.owner = new_owner.clone();
        Ok(config)
    })?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_attribute("new_owner", new_owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Project { id } => to_binary(&query_project(deps, id)?),
        QueryMsg::ListProjects { start_after, limit } => {
            to_binary(&query_projects(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
    })
}

pub fn query_project(deps: Deps, id: String) -> StdResult<ProjectResponse> {
    let project = PROJECTS.load(deps.storage, &id)?;
    Ok(project.into())
}

pub fn query_projects(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ProjectListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_deref().map(Bound::exclusive);

    let projects = PROJECTS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, project)| project.into()))
        .collect::<StdResult<Vec<ProjectResponse>>>()?;

    Ok(ProjectListResponse { projects })
}

fn assert_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            title: project.title,
            status: project.status,
            note: project.note,
            updated_by: project.updated_by.to_string(),
            updated_at: project.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::from_binary;

    use super::*;
    use crate::msg::ProjectStatus;

    #[test]
    fn owner_can_add_and_query_project() {
        let mut deps = mock_dependencies();
        let info = mock_info("owner", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), InstantiateMsg { owner: None })
            .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::AddProject {
                id: "grant-1".to_string(),
                title: "Grant milestone one".to_string(),
                status: ProjectStatus::InProgress,
                note: "Draft is ready".to_string(),
            },
        )
        .unwrap();

        let response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Project {
                id: "grant-1".to_string(),
            },
        )
        .unwrap();
        let project: ProjectResponse = from_binary(&response).unwrap();

        assert_eq!(project.title, "Grant milestone one");
        assert_eq!(project.status, ProjectStatus::InProgress);
        assert_eq!(project.updated_by, "owner");
    }

    #[test]
    fn rejects_non_owner_writes() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg { owner: None },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::AddProject {
                id: "grant-1".to_string(),
                title: "Grant milestone one".to_string(),
                status: ProjectStatus::Planned,
                note: "Trying to edit".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn owner_can_update_status_and_list_projects() {
        let mut deps = mock_dependencies();
        let info = mock_info("owner", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), InstantiateMsg { owner: None })
            .unwrap();

        for id in ["grant-1", "grant-2"] {
            execute(
                deps.as_mut(),
                mock_env(),
                info.clone(),
                ExecuteMsg::AddProject {
                    id: id.to_string(),
                    title: format!("Project {id}"),
                    status: ProjectStatus::Planned,
                    note: "Queued".to_string(),
                },
            )
            .unwrap();
        }

        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::UpdateStatus {
                id: "grant-1".to_string(),
                status: ProjectStatus::Completed,
                note: "Accepted by reviewer".to_string(),
            },
        )
        .unwrap();

        let response = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListProjects {
                start_after: None,
                limit: Some(10),
            },
        )
        .unwrap();
        let projects: ProjectListResponse = from_binary(&response).unwrap();

        assert_eq!(projects.projects.len(), 2);
        assert_eq!(projects.projects[0].id, "grant-1");
        assert_eq!(projects.projects[0].status, ProjectStatus::Completed);
        assert_eq!(projects.projects[0].note, "Accepted by reviewer");
    }

    #[test]
    fn owner_can_transfer_ownership() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg { owner: None },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::TransferOwnership {
                new_owner: "new-owner".to_string(),
            },
        )
        .unwrap();

        let response = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_binary(&response).unwrap();
        assert_eq!(config.owner, "new-owner");
    }
}
