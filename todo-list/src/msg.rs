use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_open_tasks: Option<u32>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddTask {
        title: String,
        description: Option<String>,
    },
    CompleteTask {
        id: u64,
    },
    ReopenTask {
        id: u64,
    },
    DeleteTask {
        id: u64,
    },
    UpdateConfig {
        max_open_tasks: Option<u32>,
        new_owner: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(TaskResponse)]
    Task { owner: String, id: u64 },
    #[returns(TasksResponse)]
    Tasks {
        owner: String,
        start_after: Option<u64>,
        limit: Option<u32>,
        include_completed: Option<bool>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub max_open_tasks: u32,
}

#[cw_serde]
pub struct TaskResponse {
    pub id: u64,
    pub owner: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at_height: u64,
    pub completed_at_height: Option<u64>,
}

#[cw_serde]
pub struct TasksResponse {
    pub tasks: Vec<TaskResponse>,
}
