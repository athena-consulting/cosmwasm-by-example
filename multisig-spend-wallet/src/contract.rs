#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, ProposalResponse, QueryMsg};
use crate::state::{Config, Proposal, CONFIG, PROPOSALS};

const CONTRACT_NAME: &str = "crates.io:multisig-spend-wallet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.owners.is_empty() {
        return Err(ContractError::EmptyOwners {});
    }
    if msg.threshold == 0 || msg.threshold as usize > msg.owners.len() {
        return Err(ContractError::InvalidThreshold {});
    }

    let mut owners = Vec::with_capacity(msg.owners.len());
    for owner in msg.owners {
        let owner = deps.api.addr_validate(&owner)?;
        if owners.contains(&owner) {
            return Err(ContractError::DuplicateOwner {});
        }
        owners.push(owner);
    }

    CONFIG.save(
        deps.storage,
        &Config {
            owners,
            threshold: msg.threshold,
            next_proposal_id: 1,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("threshold", msg.threshold.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProposeSpend { recipient, amount } => {
            execute_propose_spend(deps, info, recipient, amount)
        }
        ExecuteMsg::Approve { proposal_id } => execute_approve(deps, info, proposal_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Proposal { proposal_id } => to_json_binary(&query_proposal(deps, proposal_id)?),
    }
}

fn execute_propose_spend(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: cosmwasm_std::Coin,
) -> Result<Response, ContractError> {
    if amount.amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    ensure_owner(&config, &info.sender)?;

    let recipient = deps.api.addr_validate(&recipient)?;
    let proposal_id = config.next_proposal_id;
    config.next_proposal_id += 1;
    CONFIG.save(deps.storage, &config)?;

    let mut proposal = Proposal {
        id: proposal_id,
        proposer: info.sender.clone(),
        recipient,
        amount,
        approvals: vec![info.sender.clone()],
        executed: false,
    };

    let mut response = Response::new()
        .add_attribute("action", "propose_spend")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("proposer", info.sender);

    if config.threshold == 1 {
        proposal.executed = true;
        response = response.add_message(BankMsg::Send {
            to_address: proposal.recipient.to_string(),
            amount: vec![proposal.amount.clone()],
        });
    }

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;
    Ok(response)
}

fn execute_approve(
    deps: DepsMut,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_owner(&config, &info.sender)?;

    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    if proposal.executed {
        return Err(ContractError::AlreadyExecuted {});
    }
    if proposal.approvals.contains(&info.sender) {
        return Err(ContractError::AlreadyApproved {});
    }

    proposal.approvals.push(info.sender.clone());

    let mut response = Response::new()
        .add_attribute("action", "approve")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("owner", info.sender);

    if proposal.approvals.len() as u64 >= config.threshold {
        proposal.executed = true;
        response = response.add_message(BankMsg::Send {
            to_address: proposal.recipient.to_string(),
            amount: vec![proposal.amount.clone()],
        });
    }

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;
    Ok(response)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owners: config.owners.into_iter().map(String::from).collect(),
        threshold: config.threshold,
        next_proposal_id: config.next_proposal_id,
    })
}

fn query_proposal(deps: Deps, proposal_id: u64) -> StdResult<ProposalResponse> {
    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    Ok(ProposalResponse {
        id: proposal.id,
        proposer: proposal.proposer.to_string(),
        recipient: proposal.recipient.to_string(),
        amount: proposal.amount,
        approvals: proposal.approvals.into_iter().map(String::from).collect(),
        executed: proposal.executed,
    })
}

fn ensure_owner(config: &Config, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    if config.owners.contains(sender) {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}
