use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub max_text_length: u32,
    pub max_lessons_per_course: u32,
}

#[cw_serde]
pub struct Course {
    pub course_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub lessons: Vec<String>,
    pub archived: bool,
    pub created_at_height: u64,
}

#[cw_serde]
pub struct Progress {
    pub course_id: u64,
    pub learner: Addr,
    pub completed_lessons: Vec<u32>,
    pub updated_at_height: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_COURSE_ID: Item<u64> = Item::new("next_course_id");
pub const COURSE_COUNT: Item<u64> = Item::new("course_count");
pub const COURSES: Map<u64, Course> = Map::new("courses");
pub const PROGRESS: Map<(u64, &Addr), Progress> = Map::new("progress");
