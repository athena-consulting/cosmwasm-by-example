#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, CourseCountResponse, CourseResponse, ExecuteMsg, InstantiateMsg,
    ListCoursesResponse, ProgressResponse, ProgressView, QueryMsg,
};
use crate::state::{
    Config, Course, Progress, CONFIG, COURSES, COURSE_COUNT, NEXT_COURSE_ID, PROGRESS,
};

const CONTRACT_NAME: &str = "crates.io:course-progress";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_TEXT_LENGTH: u32 = 120;
const DEFAULT_MAX_LESSONS_PER_COURSE: u32 = 20;
const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        max_text_length: msg.max_text_length.unwrap_or(DEFAULT_MAX_TEXT_LENGTH),
        max_lessons_per_course: msg
            .max_lessons_per_course
            .unwrap_or(DEFAULT_MAX_LESSONS_PER_COURSE),
    };
    validate_limits(config.max_text_length, config.max_lessons_per_course)?;

    CONFIG.save(deps.storage, &config)?;
    NEXT_COURSE_ID.save(deps.storage, &1)?;
    COURSE_COUNT.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
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
        ExecuteMsg::CreateCourse {
            title,
            description,
            lessons,
        } => create_course(deps, env, info, title, description, lessons),
        ExecuteMsg::CompleteLesson {
            course_id,
            lesson_index,
        } => complete_lesson(deps, env, info, course_id, lesson_index),
        ExecuteMsg::ResetProgress { course_id } => reset_progress(deps, info, course_id),
        ExecuteMsg::ArchiveCourse { course_id } => archive_course(deps, info, course_id),
        ExecuteMsg::UpdateConfig {
            owner,
            max_text_length,
            max_lessons_per_course,
        } => update_config(deps, info, owner, max_text_length, max_lessons_per_course),
    }
}

pub fn create_course(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: Option<String>,
    lessons: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let title = validate_text("title", title, config.max_text_length)?;
    let description = validate_optional_text("description", description, config.max_text_length)?;
    let lessons = validate_lessons(
        lessons,
        config.max_text_length,
        config.max_lessons_per_course,
    )?;
    let course_id = NEXT_COURSE_ID.load(deps.storage)?;

    let course = Course {
        course_id,
        title,
        description,
        lessons,
        archived: false,
        created_at_height: env.block.height,
    };

    COURSES.save(deps.storage, course_id, &course)?;
    NEXT_COURSE_ID.save(deps.storage, &(course_id + 1))?;
    COURSE_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;

    Ok(Response::new()
        .add_attribute("action", "create_course")
        .add_attribute("course_id", course_id.to_string()))
}

pub fn complete_lesson(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    course_id: u64,
    lesson_index: u32,
) -> Result<Response, ContractError> {
    let course = COURSES
        .may_load(deps.storage, course_id)?
        .ok_or(ContractError::CourseNotFound {})?;
    if course.archived {
        return Err(ContractError::CourseArchived {});
    }
    if lesson_index as usize >= course.lessons.len() {
        return Err(ContractError::InvalidLessonIndex {});
    }

    PROGRESS.update(
        deps.storage,
        (course_id, &info.sender),
        |progress| -> Result<Progress, ContractError> {
            let mut progress = progress.unwrap_or_else(|| Progress {
                course_id,
                learner: info.sender.clone(),
                completed_lessons: vec![],
                updated_at_height: env.block.height,
            });

            if progress.completed_lessons.contains(&lesson_index) {
                return Err(ContractError::LessonAlreadyCompleted {});
            }
            progress.completed_lessons.push(lesson_index);
            progress.completed_lessons.sort_unstable();
            progress.updated_at_height = env.block.height;
            Ok(progress)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "complete_lesson")
        .add_attribute("course_id", course_id.to_string())
        .add_attribute("learner", info.sender)
        .add_attribute("lesson_index", lesson_index.to_string()))
}

pub fn reset_progress(
    deps: DepsMut,
    info: MessageInfo,
    course_id: u64,
) -> Result<Response, ContractError> {
    if !PROGRESS.has(deps.storage, (course_id, &info.sender)) {
        return Err(ContractError::ProgressNotFound {});
    }

    PROGRESS.remove(deps.storage, (course_id, &info.sender));
    Ok(Response::new()
        .add_attribute("action", "reset_progress")
        .add_attribute("course_id", course_id.to_string())
        .add_attribute("learner", info.sender))
}

pub fn archive_course(
    deps: DepsMut,
    info: MessageInfo,
    course_id: u64,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;

    COURSES.update(
        deps.storage,
        course_id,
        |course| -> Result<Course, ContractError> {
            let mut course = course.ok_or(ContractError::CourseNotFound {})?;
            course.archived = true;
            Ok(course)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "archive_course")
        .add_attribute("course_id", course_id.to_string()))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    max_text_length: Option<u32>,
    max_lessons_per_course: Option<u32>,
) -> Result<Response, ContractError> {
    assert_owner(deps.as_ref(), &info.sender)?;

    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(max_text_length) = max_text_length {
            config.max_text_length = max_text_length;
        }
        if let Some(max_lessons_per_course) = max_lessons_per_course {
            config.max_lessons_per_course = max_lessons_per_course;
        }
        validate_limits(config.max_text_length, config.max_lessons_per_course)?;
        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Course { course_id } => to_json_binary(&query_course(deps, course_id)?),
        QueryMsg::Progress { course_id, learner } => {
            to_json_binary(&query_progress(deps, course_id, learner)?)
        }
        QueryMsg::CourseCount {} => to_json_binary(&query_course_count(deps)?),
        QueryMsg::ListCourses { start_after, limit } => {
            to_json_binary(&query_list_courses(deps, start_after, limit)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _msg: crate::msg::MigrateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "migrate"))
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        max_text_length: config.max_text_length,
        max_lessons_per_course: config.max_lessons_per_course,
    })
}

fn query_course(deps: Deps, course_id: u64) -> StdResult<CourseResponse> {
    let course = COURSES.load(deps.storage, course_id)?;
    Ok(course_to_response(course))
}

fn query_progress(deps: Deps, course_id: u64, learner: String) -> StdResult<ProgressResponse> {
    let learner = deps.api.addr_validate(&learner)?;
    let progress = PROGRESS.may_load(deps.storage, (course_id, &learner))?;
    Ok(ProgressResponse {
        progress: progress.map(progress_to_response),
    })
}

fn query_course_count(deps: Deps) -> StdResult<CourseCountResponse> {
    Ok(CourseCountResponse {
        count: COURSE_COUNT.load(deps.storage)?,
    })
}

fn query_list_courses(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ListCoursesResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let courses = COURSES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|course| course.map(|(_, course)| course_to_response(course)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ListCoursesResponse { courses })
}

fn assert_owner(deps: Deps, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != *sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn course_to_response(course: Course) -> CourseResponse {
    CourseResponse {
        course_id: course.course_id,
        title: course.title,
        description: course.description,
        lessons: course.lessons,
        archived: course.archived,
        created_at_height: course.created_at_height,
    }
}

fn progress_to_response(progress: Progress) -> ProgressView {
    let completed_count = progress.completed_lessons.len() as u32;
    ProgressView {
        course_id: progress.course_id,
        learner: progress.learner.to_string(),
        completed_lessons: progress.completed_lessons,
        completed_count,
        updated_at_height: progress.updated_at_height,
    }
}

fn validate_limits(max_text_length: u32, max_lessons_per_course: u32) -> Result<(), ContractError> {
    if max_text_length == 0 || max_lessons_per_course == 0 {
        return Err(ContractError::InvalidLimit {});
    }
    Ok(())
}

fn validate_text(field: &str, value: String, max_len: u32) -> Result<String, ContractError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(ContractError::InvalidText {
            field: field.to_string(),
        });
    }
    if value.len() > max_len as usize {
        return Err(ContractError::TextTooLong {
            field: field.to_string(),
        });
    }
    Ok(value)
}

fn validate_optional_text(
    field: &str,
    value: Option<String>,
    max_len: u32,
) -> Result<Option<String>, ContractError> {
    value
        .map(|value| validate_text(field, value, max_len))
        .transpose()
}

fn validate_lessons(
    lessons: Vec<String>,
    max_text_length: u32,
    max_lessons_per_course: u32,
) -> Result<Vec<String>, ContractError> {
    if lessons.is_empty() {
        return Err(ContractError::NoLessons {});
    }
    if lessons.len() > max_lessons_per_course as usize {
        return Err(ContractError::TooManyLessons {});
    }
    lessons
        .into_iter()
        .map(|lesson| validate_text("lesson", lesson, max_text_length))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env, mock_info},
    };

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {
                max_text_length: Some(24),
                max_lessons_per_course: Some(3),
            },
        )
        .unwrap();
    }

    fn create_sample_course(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::CreateCourse {
                title: "CosmWasm 101".to_string(),
                description: Some("Intro track".to_string()),
                lessons: vec![
                    "Instantiate".to_string(),
                    "Execute".to_string(),
                    "Query".to_string(),
                ],
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_sets_defaults() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                max_text_length: None,
                max_lessons_per_course: None,
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.owner, "creator");
        assert_eq!(config.max_text_length, DEFAULT_MAX_TEXT_LENGTH);
        assert_eq!(
            config.max_lessons_per_course,
            DEFAULT_MAX_LESSONS_PER_COURSE
        );
    }

    #[test]
    fn owner_can_create_and_query_course() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_sample_course(deps.as_mut());

        let course: CourseResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Course { course_id: 1 }).unwrap())
                .unwrap();
        assert_eq!(course.title, "CosmWasm 101");
        assert_eq!(course.lessons.len(), 3);
        assert!(!course.archived);

        let count: CourseCountResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::CourseCount {}).unwrap()).unwrap();
        assert_eq!(count.count, 1);
    }

    #[test]
    fn only_owner_can_create_or_archive_courses() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let create_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateCourse {
                title: "Nope".to_string(),
                description: None,
                lessons: vec!["Lesson".to_string()],
            },
        )
        .unwrap_err();
        assert_eq!(create_err, ContractError::Unauthorized {});

        create_sample_course(deps.as_mut());
        let archive_err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::ArchiveCourse { course_id: 1 },
        )
        .unwrap_err();
        assert_eq!(archive_err, ContractError::Unauthorized {});
    }

    #[test]
    fn course_validation_rejects_bad_input() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let no_lessons = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::CreateCourse {
                title: "Empty".to_string(),
                description: None,
                lessons: vec![],
            },
        )
        .unwrap_err();
        assert_eq!(no_lessons, ContractError::NoLessons {});

        let too_many = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::CreateCourse {
                title: "Too many".to_string(),
                description: None,
                lessons: vec![
                    "One".to_string(),
                    "Two".to_string(),
                    "Three".to_string(),
                    "Four".to_string(),
                ],
            },
        )
        .unwrap_err();
        assert_eq!(too_many, ContractError::TooManyLessons {});

        let empty_title = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::CreateCourse {
                title: "   ".to_string(),
                description: None,
                lessons: vec!["Lesson".to_string()],
            },
        )
        .unwrap_err();
        assert_eq!(
            empty_title,
            ContractError::InvalidText {
                field: "title".to_string()
            }
        );
    }

    #[test]
    fn learner_can_complete_lessons_once() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_sample_course(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CompleteLesson {
                course_id: 1,
                lesson_index: 1,
            },
        )
        .unwrap();

        let progress: ProgressResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Progress {
                    course_id: 1,
                    learner: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let progress = progress.progress.unwrap();
        assert_eq!(progress.completed_lessons, vec![1]);
        assert_eq!(progress.completed_count, 1);

        let duplicate = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CompleteLesson {
                course_id: 1,
                lesson_index: 1,
            },
        )
        .unwrap_err();
        assert_eq!(duplicate, ContractError::LessonAlreadyCompleted {});
    }

    #[test]
    fn invalid_lesson_and_archived_course_are_rejected() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_sample_course(deps.as_mut());

        let invalid_lesson = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CompleteLesson {
                course_id: 1,
                lesson_index: 9,
            },
        )
        .unwrap_err();
        assert_eq!(invalid_lesson, ContractError::InvalidLessonIndex {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::ArchiveCourse { course_id: 1 },
        )
        .unwrap();

        let archived = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CompleteLesson {
                course_id: 1,
                lesson_index: 0,
            },
        )
        .unwrap_err();
        assert_eq!(archived, ContractError::CourseArchived {});
    }

    #[test]
    fn learner_can_reset_progress() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_sample_course(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CompleteLesson {
                course_id: 1,
                lesson_index: 0,
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::ResetProgress { course_id: 1 },
        )
        .unwrap();

        let progress: ProgressResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Progress {
                    course_id: 1,
                    learner: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(progress.progress.is_none());
    }

    #[test]
    fn update_config_transfers_owner() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateConfig {
                owner: Some("new_admin".to_string()),
                max_text_length: Some(40),
                max_lessons_per_course: Some(5),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.owner, "new_admin");
        assert_eq!(config.max_text_length, 40);
        assert_eq!(config.max_lessons_per_course, 5);
    }

    #[test]
    fn list_courses_supports_pagination() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        for title in ["Course A", "Course B"] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info("admin", &[]),
                ExecuteMsg::CreateCourse {
                    title: title.to_string(),
                    description: None,
                    lessons: vec!["Lesson".to_string()],
                },
            )
            .unwrap();
        }

        let first_page: ListCoursesResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListCourses {
                    start_after: None,
                    limit: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(first_page.courses.len(), 1);
        assert_eq!(first_page.courses[0].course_id, 1);

        let second_page: ListCoursesResponse = from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ListCourses {
                    start_after: Some(1),
                    limit: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(second_page.courses.len(), 1);
        assert_eq!(second_page.courses[0].course_id, 2);
    }
}
