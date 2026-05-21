use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_name_length: Option<u32>,
    pub max_metadata_length: Option<u32>,
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitScore {
        player: Option<String>,
        display_name: String,
        score: u64,
        metadata: Option<String>,
    },
    RemoveScore {
        player: String,
    },
    UpdateConfig {
        owner: Option<String>,
        max_name_length: Option<u32>,
        max_metadata_length: Option<u32>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ScoreResponse)]
    Score { player: String },
    #[returns(TopScoresResponse)]
    TopScores {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(PlayerCountResponse)]
    PlayerCount {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub max_name_length: u32,
    pub max_metadata_length: u32,
}

#[cw_serde]
pub struct ScoreResponse {
    pub player: String,
    pub display_name: String,
    pub score: u64,
    pub metadata: Option<String>,
    pub updated_at: u64,
}

#[cw_serde]
pub struct TopScoresResponse {
    pub scores: Vec<ScoreResponse>,
}

#[cw_serde]
pub struct PlayerCountResponse {
    pub count: u64,
}
