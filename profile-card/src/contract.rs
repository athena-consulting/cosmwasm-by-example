#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetProfileResponse, InstantiateMsg, ListProfilesResponse, ProfileResponse, QueryMsg,
};
use crate::state::{Profile, PROFILES};

const CONTRACT_NAME: &str = "crates.io:profile-card";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_DISPLAY_NAME_LENGTH: usize = 64;
const MAX_BIO_LENGTH: usize = 280;
const MAX_WEBSITE_LENGTH: usize = 128;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetProfile {
            display_name,
            bio,
            website,
        } => execute::set_profile(deps, info, display_name, bio, website),
        ExecuteMsg::ClearProfile {} => execute::clear_profile(deps, info),
    }
}

pub mod execute {
    use super::*;

    pub fn set_profile(
        deps: DepsMut,
        info: MessageInfo,
        display_name: String,
        bio: Option<String>,
        website: Option<String>,
    ) -> Result<Response, ContractError> {
        let profile = Profile {
            owner: info.sender.clone(),
            display_name: validate_display_name(display_name)?,
            bio: validate_optional_field(bio, MAX_BIO_LENGTH, ContractError::BioTooLong {})?,
            website: validate_optional_field(
                website,
                MAX_WEBSITE_LENGTH,
                ContractError::WebsiteTooLong {},
            )?,
        };

        PROFILES.save(deps.storage, &info.sender, &profile)?;

        Ok(Response::new()
            .add_attribute("action", "set_profile")
            .add_attribute("owner", info.sender))
    }

    pub fn clear_profile(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        PROFILES.remove(deps.storage, &info.sender);

        Ok(Response::new()
            .add_attribute("action", "clear_profile")
            .add_attribute("owner", info.sender))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProfile { address } => to_json_binary(&query::profile(deps, address)?),
        QueryMsg::ListProfiles { start_after, limit } => {
            to_json_binary(&query::profiles(deps, start_after, limit)?)
        }
    }
}

pub mod query {
    use super::*;

    pub fn profile(deps: Deps, address: String) -> StdResult<GetProfileResponse> {
        let address = deps.api.addr_validate(&address)?;
        let profile = PROFILES
            .may_load(deps.storage, &address)?
            .map(profile_to_response);

        Ok(GetProfileResponse { profile })
    }

    pub fn profiles(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<ListProfilesResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_after = start_after
            .map(|address| deps.api.addr_validate(&address))
            .transpose()?;
        let start = start_after.as_ref().map(Bound::exclusive);

        let profiles = PROFILES
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, profile)| profile_to_response(profile)))
            .collect::<StdResult<Vec<_>>>()?;

        Ok(ListProfilesResponse { profiles })
    }
}

fn validate_display_name(display_name: String) -> Result<String, ContractError> {
    let display_name = display_name.trim().to_string();

    if display_name.is_empty() {
        return Err(ContractError::EmptyDisplayName {});
    }

    if display_name.chars().count() > MAX_DISPLAY_NAME_LENGTH {
        return Err(ContractError::DisplayNameTooLong {});
    }

    Ok(display_name)
}

fn validate_optional_field(
    value: Option<String>,
    max_length: usize,
    error: ContractError,
) -> Result<Option<String>, ContractError> {
    value
        .map(|value| {
            let value = value.trim().to_string();

            if value.is_empty() {
                return Ok(None);
            }

            if value.chars().count() > max_length {
                return Err(error);
            }

            Ok(Some(value))
        })
        .transpose()
        .map(Option::flatten)
}

fn profile_to_response(profile: Profile) -> ProfileResponse {
    ProfileResponse {
        owner: profile.owner.into_string(),
        display_name: profile.display_name,
        bio: profile.bio,
        website: profile.website,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_json;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();

        assert_eq!(0, res.messages.len());
        assert_eq!("instantiate", res.attributes[0].value);
    }

    #[test]
    fn set_and_query_profile() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        let msg = ExecuteMsg::SetProfile {
            display_name: "  Alice  ".to_string(),
            bio: Some(" CosmWasm builder ".to_string()),
            website: Some(" https://example.com ".to_string()),
        };
        let res = execute(deps.as_mut(), mock_env(), mock_info("alice", &[]), msg).unwrap();

        assert_eq!("set_profile", res.attributes[0].value);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProfile {
                address: "alice".to_string(),
            },
        )
        .unwrap();
        let response: GetProfileResponse = from_json(&res).unwrap();
        let profile = response.profile.unwrap();

        assert_eq!("alice", profile.owner);
        assert_eq!("Alice", profile.display_name);
        assert_eq!(Some("CosmWasm builder".to_string()), profile.bio);
        assert_eq!(Some("https://example.com".to_string()), profile.website);
    }

    #[test]
    fn clear_profile_removes_sender_profile() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SetProfile {
                display_name: "Alice".to_string(),
                bio: None,
                website: None,
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::ClearProfile {},
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetProfile {
                address: "alice".to_string(),
            },
        )
        .unwrap();
        let response: GetProfileResponse = from_json(&res).unwrap();

        assert!(response.profile.is_none());
    }

    #[test]
    fn validates_profile_fields() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        let empty_name = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SetProfile {
                display_name: "   ".to_string(),
                bio: None,
                website: None,
            },
        );
        assert!(matches!(
            empty_name,
            Err(ContractError::EmptyDisplayName {})
        ));

        let long_bio = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SetProfile {
                display_name: "Alice".to_string(),
                bio: Some("a".repeat(MAX_BIO_LENGTH + 1)),
                website: None,
            },
        );
        assert!(matches!(long_bio, Err(ContractError::BioTooLong {})));
    }

    #[test]
    fn list_profiles_uses_pagination() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        for owner in ["alice", "bob", "carol"] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(owner, &[]),
                ExecuteMsg::SetProfile {
                    display_name: owner.to_string(),
                    bio: None,
                    website: None,
                },
            )
            .unwrap();
        }

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListProfiles {
                start_after: Some("alice".to_string()),
                limit: Some(1),
            },
        )
        .unwrap();
        let response: ListProfilesResponse = from_json(&res).unwrap();

        assert_eq!(1, response.profiles.len());
        assert_eq!("bob", response.profiles[0].owner);
    }
}
