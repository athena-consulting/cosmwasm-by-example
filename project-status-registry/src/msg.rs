use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddProject {
        id: String,
        title: String,
        status: ProjectStatus,
        note: String,
    },
    UpdateStatus {
        id: String,
        status: ProjectStatus,
        note: String,
    },
    TransferOwnership {
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(ProjectResponse)]
    Project { id: String },

    #[returns(ProjectListResponse)]
    ListProjects {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub enum ProjectStatus {
    Planned,
    InProgress,
    Blocked,
    Completed,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
}

#[cw_serde]
pub struct ProjectResponse {
    pub id: String,
    pub title: String,
    pub status: ProjectStatus,
    pub note: String,
    pub updated_by: String,
    pub updated_at: u64,
}

#[cw_serde]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectResponse>,
}
