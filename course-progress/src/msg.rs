use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_text_length: Option<u32>,
    pub max_lessons_per_course: Option<u32>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateCourse {
        title: String,
        description: Option<String>,
        lessons: Vec<String>,
    },
    CompleteLesson {
        course_id: u64,
        lesson_index: u32,
    },
    ResetProgress {
        course_id: u64,
    },
    ArchiveCourse {
        course_id: u64,
    },
    UpdateConfig {
        owner: Option<String>,
        max_text_length: Option<u32>,
        max_lessons_per_course: Option<u32>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(CourseResponse)]
    Course { course_id: u64 },
    #[returns(ProgressResponse)]
    Progress { course_id: u64, learner: String },
    #[returns(CourseCountResponse)]
    CourseCount {},
    #[returns(ListCoursesResponse)]
    ListCourses {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub max_text_length: u32,
    pub max_lessons_per_course: u32,
}

#[cw_serde]
pub struct CourseResponse {
    pub course_id: u64,
    pub title: String,
    pub description: Option<String>,
    pub lessons: Vec<String>,
    pub archived: bool,
    pub created_at_height: u64,
}

#[cw_serde]
pub struct ProgressView {
    pub course_id: u64,
    pub learner: String,
    pub completed_lessons: Vec<u32>,
    pub completed_count: u32,
    pub updated_at_height: u64,
}

#[cw_serde]
pub struct ProgressResponse {
    pub progress: Option<ProgressView>,
}

#[cw_serde]
pub struct CourseCountResponse {
    pub count: u64,
}

#[cw_serde]
pub struct ListCoursesResponse {
    pub courses: Vec<CourseResponse>,
}

#[cw_serde]
pub struct MigrateMsg {}
