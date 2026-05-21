#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    AnswerResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, ListQuizzesResponse,
    PlayerStatsResponse, QueryMsg, QuizResponse,
};
use crate::state::{Answer, Config, Quiz, ANSWERS, CONFIG, NEXT_QUIZ_ID, PLAYER_STATS, QUIZZES};

const CONTRACT_NAME: &str = "crates.io:quiz";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MAX_TEXT_LENGTH: u32 = 160;
const DEFAULT_MAX_OPTIONS: u32 = 8;
const DEFAULT_LIMIT: u32 = 10;
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
        max_options: msg.max_options.unwrap_or(DEFAULT_MAX_OPTIONS),
    };
    validate_limits(config.max_text_length, config.max_options)?;

    CONFIG.save(deps.storage, &config)?;
    NEXT_QUIZ_ID.save(deps.storage, &1)?;

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
        ExecuteMsg::CreateQuiz {
            title,
            question,
            options,
            correct_option,
            closes_at,
        } => create_quiz(
            deps,
            env,
            info,
            title,
            question,
            options,
            correct_option,
            closes_at,
        ),
        ExecuteMsg::SubmitAnswer {
            quiz_id,
            selected_option,
        } => submit_answer(deps, env, info, quiz_id, selected_option),
        ExecuteMsg::CloseQuiz { quiz_id } => close_quiz(deps, info, quiz_id),
        ExecuteMsg::UpdateConfig {
            owner,
            max_text_length,
            max_options,
        } => update_config(deps, info, owner, max_text_length, max_options),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_quiz(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    question: String,
    options: Vec<String>,
    correct_option: u32,
    closes_at: Option<u64>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    validate_text(&title, config.max_text_length)?;
    validate_text(&question, config.max_text_length)?;
    validate_options(
        &options,
        correct_option,
        config.max_text_length,
        config.max_options,
    )?;

    let id = NEXT_QUIZ_ID.load(deps.storage)?;
    let quiz = Quiz {
        id,
        title,
        question,
        options,
        correct_option,
        closes_at,
        closed: false,
        created_at: env.block.height,
    };

    QUIZZES.save(deps.storage, id, &quiz)?;
    NEXT_QUIZ_ID.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_quiz")
        .add_attribute("quiz_id", id.to_string()))
}

pub fn submit_answer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    quiz_id: u64,
    selected_option: u32,
) -> Result<Response, ContractError> {
    let quiz = QUIZZES
        .may_load(deps.storage, quiz_id)?
        .ok_or(ContractError::QuizNotFound {})?;

    if quiz.closed
        || quiz
            .closes_at
            .map(|height| env.block.height >= height)
            .unwrap_or(false)
    {
        return Err(ContractError::QuizClosed {});
    }
    if selected_option as usize >= quiz.options.len() {
        return Err(ContractError::InvalidSelectedOption {});
    }
    if ANSWERS
        .may_load(deps.storage, (quiz_id, &info.sender))?
        .is_some()
    {
        return Err(ContractError::AlreadyAnswered {});
    }

    let correct = selected_option == quiz.correct_option;
    let answer = Answer {
        quiz_id,
        player: info.sender.clone(),
        selected_option,
        correct,
        answered_at: env.block.height,
    };
    ANSWERS.save(deps.storage, (quiz_id, &info.sender), &answer)?;

    PLAYER_STATS.update(deps.storage, &info.sender, |stats| -> StdResult<_> {
        let mut stats = stats.unwrap_or_default();
        stats.answered += 1;
        if correct {
            stats.correct += 1;
        }
        Ok(stats)
    })?;

    Ok(Response::new()
        .add_attribute("action", "submit_answer")
        .add_attribute("quiz_id", quiz_id.to_string())
        .add_attribute("player", info.sender)
        .add_attribute("correct", correct.to_string()))
}

pub fn close_quiz(
    deps: DepsMut,
    info: MessageInfo,
    quiz_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    QUIZZES.update(deps.storage, quiz_id, |quiz| -> Result<_, ContractError> {
        let mut quiz = quiz.ok_or(ContractError::QuizNotFound {})?;
        quiz.closed = true;
        Ok(quiz)
    })?;

    Ok(Response::new()
        .add_attribute("action", "close_quiz")
        .add_attribute("quiz_id", quiz_id.to_string()))
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    max_text_length: Option<u32>,
    max_options: Option<u32>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(max_text_length) = max_text_length {
            config.max_text_length = max_text_length;
        }
        if let Some(max_options) = max_options {
            config.max_options = max_options;
        }
        validate_limits(config.max_text_length, config.max_options)?;

        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Quiz { quiz_id } => to_json_binary(&query_quiz(deps, quiz_id)?),
        QueryMsg::Answer { quiz_id, player } => {
            to_json_binary(&query_answer(deps, quiz_id, player)?)
        }
        QueryMsg::PlayerStats { player } => to_json_binary(&query_player_stats(deps, player)?),
        QueryMsg::ListQuizzes { start_after, limit } => {
            to_json_binary(&query_list_quizzes(deps, start_after, limit)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        max_text_length: config.max_text_length,
        max_options: config.max_options,
    })
}

pub fn query_quiz(deps: Deps, quiz_id: u64) -> StdResult<QuizResponse> {
    let quiz = QUIZZES.load(deps.storage, quiz_id)?;
    Ok(quiz_response(quiz))
}

pub fn query_answer(deps: Deps, quiz_id: u64, player: String) -> StdResult<AnswerResponse> {
    let player_addr = deps.api.addr_validate(&player)?;
    let answer = ANSWERS.load(deps.storage, (quiz_id, &player_addr))?;
    Ok(answer_response(answer))
}

pub fn query_player_stats(deps: Deps, player: String) -> StdResult<PlayerStatsResponse> {
    let player_addr = deps.api.addr_validate(&player)?;
    let stats = PLAYER_STATS
        .may_load(deps.storage, &player_addr)?
        .unwrap_or_default();
    Ok(PlayerStatsResponse {
        player,
        answered: stats.answered,
        correct: stats.correct,
    })
}

pub fn query_list_quizzes(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ListQuizzesResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let quizzes = QUIZZES
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, quiz)| quiz_response(quiz)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ListQuizzesResponse { quizzes })
}

fn quiz_response(quiz: Quiz) -> QuizResponse {
    QuizResponse {
        id: quiz.id,
        title: quiz.title,
        question: quiz.question,
        options: quiz.options,
        correct_option: quiz.correct_option,
        closes_at: quiz.closes_at,
        closed: quiz.closed,
        created_at: quiz.created_at,
    }
}

fn answer_response(answer: Answer) -> AnswerResponse {
    AnswerResponse {
        quiz_id: answer.quiz_id,
        player: answer.player.to_string(),
        selected_option: answer.selected_option,
        correct: answer.correct,
        answered_at: answer.answered_at,
    }
}

fn validate_limits(max_text_length: u32, max_options: u32) -> Result<(), ContractError> {
    if max_text_length == 0 || max_options < 2 {
        return Err(ContractError::InvalidLimit {});
    }
    Ok(())
}

fn validate_text(value: &str, max_text_length: u32) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        return Err(ContractError::EmptyText {});
    }
    if value.len() > max_text_length as usize {
        return Err(ContractError::TextTooLong {});
    }
    Ok(())
}

fn validate_options(
    options: &[String],
    correct_option: u32,
    max_text_length: u32,
    max_options: u32,
) -> Result<(), ContractError> {
    if options.len() < 2 {
        return Err(ContractError::NotEnoughOptions {});
    }
    if options.len() > max_options as usize {
        return Err(ContractError::TooManyOptions {});
    }
    if correct_option as usize >= options.len() {
        return Err(ContractError::InvalidCorrectOption {});
    }
    for option in options {
        validate_text(option, max_text_length)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg {
                max_text_length: None,
                max_options: None,
            },
        )
        .unwrap();
    }

    fn create_basic_quiz(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CreateQuiz {
                title: "CosmWasm basics".to_string(),
                question: "Which message creates contract state?".to_string(),
                options: vec![
                    "Instantiate".to_string(),
                    "Execute".to_string(),
                    "Query".to_string(),
                ],
                correct_option: 0,
                closes_at: None,
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_and_query_config() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!("owner", config.owner);
        assert_eq!(DEFAULT_MAX_TEXT_LENGTH, config.max_text_length);
        assert_eq!(DEFAULT_MAX_OPTIONS, config.max_options);
    }

    #[test]
    fn owner_creates_and_lists_quiz() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_basic_quiz(deps.as_mut());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Quiz { quiz_id: 1 }).unwrap();
        let quiz: QuizResponse = from_json(&res).unwrap();
        assert_eq!(1, quiz.id);
        assert_eq!("CosmWasm basics", quiz.title);
        assert_eq!(0, quiz.correct_option);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListQuizzes {
                start_after: None,
                limit: Some(10),
            },
        )
        .unwrap();
        let quizzes: ListQuizzesResponse = from_json(&res).unwrap();
        assert_eq!(1, quizzes.quizzes.len());
    }

    #[test]
    fn player_submits_correct_answer_and_stats_update() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_basic_quiz(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SubmitAnswer {
                quiz_id: 1,
                selected_option: 0,
            },
        )
        .unwrap();

        let answer: AnswerResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Answer {
                    quiz_id: 1,
                    player: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(answer.correct);

        let stats: PlayerStatsResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::PlayerStats {
                    player: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(1, stats.answered);
        assert_eq!(1, stats.correct);
    }

    #[test]
    fn wrong_answer_is_recorded_without_correct_credit() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_basic_quiz(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SubmitAnswer {
                quiz_id: 1,
                selected_option: 2,
            },
        )
        .unwrap();

        let stats: PlayerStatsResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::PlayerStats {
                    player: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(1, stats.answered);
        assert_eq!(0, stats.correct);
    }

    #[test]
    fn duplicate_answers_are_rejected() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_basic_quiz(deps.as_mut());

        let msg = ExecuteMsg::SubmitAnswer {
            quiz_id: 1,
            selected_option: 0,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            msg.clone(),
        )
        .unwrap();

        let err = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap_err();
        assert_eq!(ContractError::AlreadyAnswered {}, err);
    }

    #[test]
    fn close_quiz_prevents_answers() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_basic_quiz(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CloseQuiz { quiz_id: 1 },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SubmitAnswer {
                quiz_id: 1,
                selected_option: 0,
            },
        )
        .unwrap_err();
        assert_eq!(ContractError::QuizClosed {}, err);
    }

    #[test]
    fn unauthorized_create_and_close_are_rejected() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateQuiz {
                title: "Nope".to_string(),
                question: "Nope?".to_string(),
                options: vec!["A".to_string(), "B".to_string()],
                correct_option: 0,
                closes_at: None,
            },
        )
        .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, err);

        create_basic_quiz(deps.as_mut());
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CloseQuiz { quiz_id: 1 },
        )
        .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, err);
    }

    #[test]
    fn validation_rejects_invalid_quizzes_and_config() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CreateQuiz {
                title: "Bad".to_string(),
                question: "Pick one".to_string(),
                options: vec!["Only one".to_string()],
                correct_option: 0,
                closes_at: None,
            },
        )
        .unwrap_err();
        assert_eq!(ContractError::NotEnoughOptions {}, err);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                owner: None,
                max_text_length: Some(0),
                max_options: None,
            },
        )
        .unwrap_err();
        assert_eq!(ContractError::InvalidLimit {}, err);
    }
}
