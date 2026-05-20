#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ContactResponse, ContactsResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use crate::state::{Config, Contact, CONFIG, CONTACTS};

const CONTRACT_NAME: &str = "crates.io:address-book";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;
const MAX_NOTE_LENGTH: usize = 280;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender.clone(),
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddContact {
            name,
            address,
            note,
        } => execute_add_contact(deps, info, name, address, note),
        ExecuteMsg::RemoveContact { name } => execute_remove_contact(deps, info, name),
        ExecuteMsg::TransferOwnership { new_owner } => {
            execute_transfer_ownership(deps, info, new_owner)
        }
    }
}

pub fn execute_add_contact(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    address: String,
    note: Option<String>,
) -> Result<Response, ContractError> {
    ensure_owner(deps.as_ref(), &info)?;
    let name = normalize_name(name)?;
    ensure_note_length(&note)?;

    if CONTACTS.may_load(deps.storage, name.clone())?.is_some() {
        return Err(ContractError::ContactExists {});
    }

    let contact = Contact {
        address: deps.api.addr_validate(&address)?,
        note,
    };
    CONTACTS.save(deps.storage, name.clone(), &contact)?;

    Ok(Response::new()
        .add_attribute("action", "add_contact")
        .add_attribute("name", name)
        .add_attribute("address", contact.address))
}

pub fn execute_remove_contact(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    ensure_owner(deps.as_ref(), &info)?;
    let name = normalize_name(name)?;

    CONTACTS.remove(deps.storage, name.clone());

    Ok(Response::new()
        .add_attribute("action", "remove_contact")
        .add_attribute("name", name))
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    ensure_owner(deps.as_ref(), &info)?;
    let new_owner = deps.api.addr_validate(&new_owner)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: new_owner.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "transfer_ownership")
        .add_attribute("new_owner", new_owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Contact { name } => to_json_binary(&query_contact(deps, name)?),
        QueryMsg::Contacts { start_after, limit } => {
            to_json_binary(&query_contacts(deps, start_after, limit)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
    })
}

fn query_contact(deps: Deps, name: String) -> StdResult<ContactResponse> {
    let name = normalize_name_for_query(name);
    let contact = CONTACTS.load(deps.storage, name.clone())?;

    Ok(contact_response(name, contact))
}

fn query_contacts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ContactsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after
        .map(normalize_name_for_query)
        .map(Bound::exclusive);

    let contacts = CONTACTS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(name, contact)| contact_response(name, contact)))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(ContactsResponse { contacts })
}

fn ensure_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

fn normalize_name(name: String) -> Result<String, ContractError> {
    let name = normalize_name_for_query(name);

    if name.is_empty() {
        return Err(ContractError::EmptyName {});
    }

    Ok(name)
}

fn normalize_name_for_query(name: String) -> String {
    name.trim().to_ascii_lowercase()
}

fn ensure_note_length(note: &Option<String>) -> Result<(), ContractError> {
    if note.as_ref().map(|note| note.len()).unwrap_or_default() > MAX_NOTE_LENGTH {
        return Err(ContractError::NoteTooLong {});
    }

    Ok(())
}

fn contact_response(name: String, contact: Contact) -> ContactResponse {
    ContactResponse {
        name,
        address: contact.address.to_string(),
        note: contact.note,
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};

    use super::*;

    #[test]
    fn instantiate_uses_sender_as_default_owner() {
        let mut deps = mock_dependencies();
        let info = mock_info("owner", &coins(1, "uatom"));

        instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            InstantiateMsg { owner: None },
        )
        .unwrap();

        let response: ConfigResponse =
            from_json(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(response.owner, "owner");
    }

    #[test]
    fn owner_can_add_and_query_contact() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg { owner: None },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::AddContact {
                name: "Validator One".to_string(),
                address: "validator".to_string(),
                note: Some("primary validator".to_string()),
            },
        )
        .unwrap();

        let response: ContactResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Contact {
                    name: "validator one".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(response.name, "validator one");
        assert_eq!(response.address, "validator");
        assert_eq!(response.note, Some("primary validator".to_string()));
    }

    #[test]
    fn non_owner_cannot_add_contact() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg { owner: None },
        )
        .unwrap();

        let error = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("visitor", &[]),
            ExecuteMsg::AddContact {
                name: "friend".to_string(),
                address: "friend".to_string(),
                note: None,
            },
        )
        .unwrap_err();

        assert_eq!(error, ContractError::Unauthorized {});
    }

    #[test]
    fn contacts_query_is_paginated() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg { owner: None },
        )
        .unwrap();

        for name in ["alice", "bob", "carol"] {
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info("owner", &[]),
                ExecuteMsg::AddContact {
                    name: name.to_string(),
                    address: format!("{name}-addr"),
                    note: None,
                },
            )
            .unwrap();
        }

        let response: ContactsResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Contacts {
                    start_after: Some("alice".to_string()),
                    limit: Some(2),
                },
            )
            .unwrap(),
        )
        .unwrap();

        let names: Vec<String> = response
            .contacts
            .into_iter()
            .map(|contact| contact.name)
            .collect();
        assert_eq!(names, vec!["bob".to_string(), "carol".to_string()]);
    }

    #[test]
    fn owner_can_remove_contact() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            InstantiateMsg { owner: None },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::AddContact {
                name: "friend".to_string(),
                address: "friend".to_string(),
                note: None,
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::RemoveContact {
                name: "friend".to_string(),
            },
        )
        .unwrap();

        let response: ContactsResponse = from_json(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Contacts {
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert!(response.contacts.is_empty());
    }
}
