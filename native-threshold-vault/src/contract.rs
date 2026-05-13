#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, ProposalResponse, QueryMsg};
use crate::state::{Config, Proposal, CONFIG, PROPOSALS};

const CONTRACT_NAME: &str = "crates.io:native-threshold-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.denom.trim().is_empty() {
        return Err(ContractError::EmptyDenom {});
    }
    if msg.members.is_empty() {
        return Err(ContractError::NoMembers {});
    }

    let mut members = Vec::with_capacity(msg.members.len());
    for member in msg.members {
        let member = deps.api.addr_validate(&member)?;
        if members.iter().any(|existing| existing == &member) {
            return Err(ContractError::DuplicateMember {});
        }
        members.push(member);
    }

    if msg.threshold == 0 || msg.threshold as usize > members.len() {
        return Err(ContractError::InvalidThreshold {});
    }

    let config = Config {
        members,
        threshold: msg.threshold,
        denom: msg.denom,
        next_proposal_id: 1,
    };
    CONFIG.save(deps.storage, &config)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("member_count", config.members.len().to_string())
        .add_attribute("threshold", config.threshold.to_string())
        .add_attribute("denom", config.denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => execute_deposit(deps, info),
        ExecuteMsg::ProposeSpend {
            recipient,
            amount,
            description,
        } => execute_propose_spend(deps, info, recipient, amount, description),
        ExecuteMsg::Approve { proposal_id } => execute_approve(deps, info, proposal_id),
        ExecuteMsg::Execute { proposal_id } => execute_proposal(deps, env, info, proposal_id),
    }
}

fn execute_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let paid = must_pay_configured_denom(&info, &config)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender)
        .add_attribute("amount", paid.amount)
        .add_attribute("denom", paid.denom))
}

fn execute_propose_spend(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
    description: String,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::ZeroAmount {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    ensure_member(&config, &info.sender)?;

    let recipient = deps.api.addr_validate(&recipient)?;
    let proposal_id = config.next_proposal_id;
    config.next_proposal_id += 1;
    CONFIG.save(deps.storage, &config)?;

    let proposal = Proposal {
        id: proposal_id,
        proposer: info.sender.clone(),
        recipient,
        amount,
        description,
        approvals: vec![info.sender.clone()],
        executed: false,
    };
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "propose_spend")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("proposer", info.sender)
        .add_attribute("amount", amount)
        .add_attribute("approval_count", "1"))
}

fn execute_approve(
    deps: DepsMut,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_member(&config, &info.sender)?;

    let mut proposal = PROPOSALS
        .may_load(deps.storage, proposal_id)?
        .ok_or(ContractError::ProposalNotFound {})?;
    if proposal.executed {
        return Err(ContractError::AlreadyExecuted {});
    }
    if proposal
        .approvals
        .iter()
        .any(|member| member == &info.sender)
    {
        return Err(ContractError::AlreadyApproved {});
    }

    proposal.approvals.push(info.sender.clone());
    let approval_count = proposal.approvals.len();
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("approver", info.sender)
        .add_attribute("approval_count", approval_count.to_string()))
}

fn execute_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_member(&config, &info.sender)?;

    let mut proposal = PROPOSALS
        .may_load(deps.storage, proposal_id)?
        .ok_or(ContractError::ProposalNotFound {})?;
    if proposal.executed {
        return Err(ContractError::AlreadyExecuted {});
    }
    if proposal.approvals.len() < config.threshold as usize {
        return Err(ContractError::ThresholdNotMet {});
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?;
    if balance.amount < proposal.amount {
        return Err(ContractError::InsufficientFunds {});
    }

    proposal.executed = true;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    let msg = BankMsg::Send {
        to_address: proposal.recipient.to_string(),
        amount: vec![Coin::new(proposal.amount.u128(), config.denom.clone())],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("recipient", proposal.recipient)
        .add_attribute("amount", proposal.amount)
        .add_attribute("denom", config.denom))
}

fn must_pay_configured_denom(info: &MessageInfo, config: &Config) -> Result<Coin, ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidDeposit {});
    }

    let paid = info.funds[0].clone();
    if paid.denom != config.denom || paid.amount.is_zero() {
        return Err(ContractError::InvalidDeposit {});
    }

    Ok(paid)
}

fn ensure_member(config: &Config, sender: &cosmwasm_std::Addr) -> Result<(), ContractError> {
    if config.members.iter().any(|member| member == sender) {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Proposal { proposal_id } => to_binary(&query_proposal(deps, proposal_id)?),
    }
}

fn query_config(deps: Deps) -> Result<ConfigResponse, StdError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        members: config
            .members
            .into_iter()
            .map(|member| member.to_string())
            .collect(),
        threshold: config.threshold,
        denom: config.denom,
        next_proposal_id: config.next_proposal_id,
    })
}

fn query_proposal(deps: Deps, proposal_id: u64) -> Result<ProposalResponse, StdError> {
    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    Ok(ProposalResponse {
        id: proposal.id,
        proposer: proposal.proposer.to_string(),
        recipient: proposal.recipient.to_string(),
        amount: proposal.amount,
        description: proposal.description,
        approvals: proposal
            .approvals
            .into_iter()
            .map(|member| member.to_string())
            .collect(),
        executed: proposal.executed,
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, BankMsg, CosmosMsg, Uint128};

    use super::*;

    fn instantiate_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            members: vec!["alice".to_string(), "bob".to_string(), "carol".to_string()],
            threshold: 2,
            denom: "uatom".to_string(),
        };
        instantiate(deps, mock_env(), mock_info("creator", &[]), msg).unwrap();
    }

    #[test]
    fn instantiate_rejects_invalid_thresholds_and_duplicates() {
        let mut deps = mock_dependencies();
        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                members: vec!["alice".to_string(), "alice".to_string()],
                threshold: 1,
                denom: "uatom".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::DuplicateMember {});

        let mut deps = mock_dependencies();
        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                members: vec!["alice".to_string()],
                threshold: 2,
                denom: "uatom".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidThreshold {});
    }

    #[test]
    fn deposit_requires_single_positive_configured_denom() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("donor", &coins(0, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDeposit {});

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("donor", &coins(10, "uosmo")),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDeposit {});

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("donor", &coins(10, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "deposit");
    }

    #[test]
    fn only_members_can_create_and_approve_proposals() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("mallory", &[]),
            ExecuteMsg::ProposeSpend {
                recipient: "vendor".to_string(),
                amount: Uint128::new(25),
                description: "pay invoice".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::ProposeSpend {
                recipient: "vendor".to_string(),
                amount: Uint128::new(25),
                description: "pay invoice".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("mallory", &[]),
            ExecuteMsg::Approve { proposal_id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn proposer_auto_approves_and_duplicate_approval_fails() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::ProposeSpend {
                recipient: "vendor".to_string(),
                amount: Uint128::new(25),
                description: "pay invoice".to_string(),
            },
        )
        .unwrap();

        let proposal = query_proposal(deps.as_ref(), 1).unwrap();
        assert_eq!(proposal.approvals, vec!["alice".to_string()]);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Approve { proposal_id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AlreadyApproved {});

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Approve { proposal_id: 1 },
        )
        .unwrap();
        assert_eq!(res.attributes[3].value, "2");
    }

    #[test]
    fn execution_requires_threshold_and_contract_balance() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        let env = mock_env();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::ProposeSpend {
                recipient: "vendor".to_string(),
                amount: Uint128::new(25),
                description: "pay invoice".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::Execute { proposal_id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::ThresholdNotMet {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("bob", &[]),
            ExecuteMsg::Approve { proposal_id: 1 },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::Execute { proposal_id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InsufficientFunds {});

        deps.querier
            .update_balance(env.contract.address.clone(), coins(100, "uatom"));
        let res = execute(
            deps.as_mut(),
            env,
            mock_info("carol", &[]),
            ExecuteMsg::Execute { proposal_id: 1 },
        )
        .unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "vendor".to_string(),
                amount: coins(25, "uatom"),
            })
        );
    }

    #[test]
    fn executed_proposal_cannot_be_approved_or_executed_again() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        let env = mock_env();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::ProposeSpend {
                recipient: "vendor".to_string(),
                amount: Uint128::new(25),
                description: "pay invoice".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("bob", &[]),
            ExecuteMsg::Approve { proposal_id: 1 },
        )
        .unwrap();
        deps.querier
            .update_balance(env.contract.address.clone(), coins(100, "uatom"));
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("alice", &[]),
            ExecuteMsg::Execute { proposal_id: 1 },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("carol", &[]),
            ExecuteMsg::Approve { proposal_id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AlreadyExecuted {});

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::Execute { proposal_id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AlreadyExecuted {});
    }
}
