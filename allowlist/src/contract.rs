#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MemberResponse, QueryMsg};
use crate::state::{ADMIN, ALLOWLIST};

const CONTRACT_NAME: &str = "crates.io:allowlist";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = match msg.admin {
        Some(admin) => deps.api.addr_validate(&admin)?,
        None => info.sender,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    ADMIN.save(deps.storage, &admin)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMember { address, label } => execute::add_member(deps, info, address, label),
        ExecuteMsg::RemoveMember { address } => execute::remove_member(deps, info, address),
    }
}

pub mod execute {
    use super::*;

    fn ensure_admin(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
        let admin = ADMIN.load(deps.storage)?;
        if info.sender != admin {
            return Err(ContractError::Unauthorized {});
        }
        Ok(())
    }

    pub fn add_member(
        deps: DepsMut,
        info: MessageInfo,
        address: String,
        label: String,
    ) -> Result<Response, ContractError> {
        ensure_admin(deps.as_ref(), &info)?;
        let member = deps.api.addr_validate(&address)?;
        ALLOWLIST.save(deps.storage, &member, &label)?;

        Ok(Response::new()
            .add_attribute("action", "add_member")
            .add_attribute("member", member)
            .add_attribute("label", label))
    }

    pub fn remove_member(
        deps: DepsMut,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        ensure_admin(deps.as_ref(), &info)?;
        let member = deps.api.addr_validate(&address)?;
        ALLOWLIST.remove(deps.storage, &member);

        Ok(Response::new()
            .add_attribute("action", "remove_member")
            .add_attribute("member", member))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsMember { address } => to_binary(&query::is_member(deps, address)?),
    }
}

pub mod query {
    use super::*;

    pub fn is_member(deps: Deps, address: String) -> StdResult<MemberResponse> {
        let member = deps.api.addr_validate(&address)?;
        let label = ALLOWLIST.may_load(deps.storage, &member)?;
        Ok(MemberResponse {
            is_member: label.is_some(),
            label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Addr};

    fn setup() -> cosmwasm_std::OwnedDeps<
        cosmwasm_std::MemoryStorage,
        cosmwasm_std::testing::MockApi,
        cosmwasm_std::testing::MockQuerier,
    > {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            InstantiateMsg { admin: None },
        )
        .unwrap();
        deps
    }

    #[test]
    fn admin_can_add_member_with_label() {
        let mut deps = setup();
        let info = mock_info("creator", &[]);

        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::AddMember {
                address: "member".to_string(),
                label: "early tester".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsMember {
                address: "member".to_string(),
            },
        )
        .unwrap();
        let member: MemberResponse = from_binary(&res).unwrap();
        assert!(member.is_member);
        assert_eq!(member.label, Some("early tester".to_string()));
    }

    #[test]
    fn non_admin_cannot_add_member() {
        let mut deps = setup();
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::AddMember {
                address: "member".to_string(),
                label: "early tester".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn admin_can_remove_member() {
        let mut deps = setup();
        let info = mock_info("creator", &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::AddMember {
                address: "member".to_string(),
                label: "early tester".to_string(),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::RemoveMember {
                address: "member".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::IsMember {
                address: "member".to_string(),
            },
        )
        .unwrap();
        let member: MemberResponse = from_binary(&res).unwrap();
        assert!(!member.is_member);
        assert_eq!(member.label, None);
    }

    #[test]
    fn optional_admin_can_be_set_on_instantiation() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                admin: Some("governance".to_string()),
            },
        )
        .unwrap();

        let admin = ADMIN.load(&deps.storage).unwrap();
        assert_eq!(admin, Addr::unchecked("governance"));
    }
}
