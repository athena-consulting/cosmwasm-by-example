#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ProposalResponse};
use crate::state::{Config, Proposal, CONFIG, PROPOSALS, PROPOSAL_COUNT};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.threshold == 0 || msg.threshold as usize > msg.signers.len() {
        return Err(ContractError::InvalidThreshold {});
    }

    let config = Config {
        signers: msg.signers,
        threshold: msg.threshold,
    };
    CONFIG.save(deps.storage, &config)?;
    PROPOSAL_COUNT.save(deps.storage, &0)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Propose { msg } => execute_propose(deps, info, msg),
        ExecuteMsg::Approve { proposal_id } => execute_approve(deps, info, proposal_id),
        ExecuteMsg::Execute { proposal_id } => execute_exec(deps, proposal_id),
    }
}

pub fn execute_propose(
    deps: DepsMut,
    info: MessageInfo,
    msg: CosmosMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.signers.contains(&info.sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }

    let id = PROPOSAL_COUNT.load(deps.storage)?;
    PROPOSAL_COUNT.save(deps.storage, &(id + 1))?;

    let prop = Proposal {
        msg,
        approvals: vec![info.sender.to_string()],
    };
    PROPOSALS.save(deps.storage, id, &prop)?;

    Ok(Response::new().add_attribute("action", "propose"))
}

pub fn execute_approve(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.signers.contains(&info.sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }

    let mut prop = PROPOSALS.load(deps.storage, id)?;
    if prop.approvals.contains(&info.sender.to_string()) {
        return Err(ContractError::AlreadyApproved {});
    }

    prop.approvals.push(info.sender.to_string());
    PROPOSALS.save(deps.storage, id, &prop)?;

    Ok(Response::new().add_attribute("action", "approve"))
}

pub fn execute_exec(deps: DepsMut, id: u64) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let prop = PROPOSALS.load(deps.storage, id)?;

    if (prop.approvals.len() as u64) < config.threshold {
        return Err(ContractError::NotEnoughApprovals {});
    }

    PROPOSALS.remove(deps.storage, id);

    Ok(Response::new()
        .add_message(prop.msg)
        .add_attribute("action", "execute"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Proposal { id } => to_binary(&query_prop(deps, id)?),
    }
}

fn query_prop(deps: Deps, id: u64) -> StdResult<ProposalResponse> {
    let prop = PROPOSALS.load(deps.storage, id)?;
    Ok(ProposalResponse {
        approvals: prop.approvals.len() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, BankMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            signers: vec!["a".to_string(), "b".to_string()],
            threshold: 2,
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn reject_invalid_thresholds() {
        let mut deps = mock_dependencies();
        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                signers: vec!["a".to_string(), "b".to_string()],
                threshold: 0,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidThreshold {});

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                signers: vec!["a".to_string(), "b".to_string()],
                threshold: 3,
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidThreshold {});
    }

    #[test]
    fn executed_proposal_cannot_be_replayed() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                signers: vec!["a".to_string(), "b".to_string()],
                threshold: 2,
            },
        )
        .unwrap();

        let send_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: "recipient".to_string(),
            amount: coins(1, "earth"),
        });
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("a", &[]),
            ExecuteMsg::Propose { msg: send_msg },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("b", &[]),
            ExecuteMsg::Approve { proposal_id: 0 },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("a", &[]),
            ExecuteMsg::Execute { proposal_id: 0 },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("a", &[]),
            ExecuteMsg::Execute { proposal_id: 0 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Std(_)));
    }
}
