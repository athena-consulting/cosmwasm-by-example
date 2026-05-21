use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub max_text_length: u32,
    pub max_options: u32,
}

#[cw_serde]
pub struct Quiz {
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
pub struct Answer {
    pub quiz_id: u64,
    pub player: Addr,
    pub selected_option: u32,
    pub correct: bool,
    pub answered_at: u64,
}

#[cw_serde]
#[derive(Default)]
pub struct PlayerStats {
    pub answered: u64,
    pub correct: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_QUIZ_ID: Item<u64> = Item::new("next_quiz_id");
pub const QUIZZES: Map<u64, Quiz> = Map::new("quizzes");
pub const ANSWERS: Map<(u64, &Addr), Answer> = Map::new("answers");
pub const PLAYER_STATS: Map<&Addr, PlayerStats> = Map::new("player_stats");
