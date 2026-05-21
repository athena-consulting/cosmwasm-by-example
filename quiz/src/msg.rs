use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_text_length: Option<u32>,
    pub max_options: Option<u32>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateQuiz {
        title: String,
        question: String,
        options: Vec<String>,
        correct_option: u32,
        closes_at: Option<u64>,
    },
    SubmitAnswer {
        quiz_id: u64,
        selected_option: u32,
    },
    CloseQuiz {
        quiz_id: u64,
    },
    UpdateConfig {
        owner: Option<String>,
        max_text_length: Option<u32>,
        max_options: Option<u32>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(QuizResponse)]
    Quiz { quiz_id: u64 },
    #[returns(AnswerResponse)]
    Answer { quiz_id: u64, player: String },
    #[returns(PlayerStatsResponse)]
    PlayerStats { player: String },
    #[returns(ListQuizzesResponse)]
    ListQuizzes {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub max_text_length: u32,
    pub max_options: u32,
}

#[cw_serde]
pub struct QuizResponse {
    pub id: u64,
    pub title: String,
    pub question: String,
    pub options: Vec<String>,
    pub correct_option: u32,
    pub closes_at: Option<u64>,
    pub closed: bool,
    pub created_at: u64,
}

#[cw_serde]
pub struct AnswerResponse {
    pub quiz_id: u64,
    pub player: String,
    pub selected_option: u32,
    pub correct: bool,
    pub answered_at: u64,
}

#[cw_serde]
pub struct PlayerStatsResponse {
    pub player: String,
    pub answered: u64,
    pub correct: u64,
}

#[cw_serde]
pub struct ListQuizzesResponse {
    pub quizzes: Vec<QuizResponse>,
}
