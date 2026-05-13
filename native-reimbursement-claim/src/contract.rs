#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ClaimResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, ReimbursementClaim, CLAIMS, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-reimbursement-claim";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let denom = msg.denom.trim().to_string();
    if denom.is_empty() {
        return Err(ContractError::EmptyDenom {});
    }

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            denom: denom.clone(),
            balance: Uint128::zero(),
            next_claim_id: 1,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Fund {} => execute_fund(deps, info),
        ExecuteMsg::ApproveClaim {
            claimant,
            amount,
            description,
        } => execute_approve_claim(deps, info, claimant, amount, description),
        ExecuteMsg::Claim { claim_id } => execute_claim(deps, info, claim_id),
        ExecuteMsg::CancelClaim { claim_id } => execute_cancel_claim(deps, info, claim_id),
        ExecuteMsg::WithdrawExcess { recipient, amount } => {
            execute_withdraw_excess(deps, info, recipient, amount)
        }
    }
}

fn execute_fund(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    config.balance = config
        .balance
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "fund")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("balance", config.balance))
}

fn execute_approve_claim(
    deps: DepsMut,
    info: MessageInfo,
    claimant: String,
    amount: Uint128,
    description: Option<String>,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let claimant = deps.api.addr_validate(&claimant)?;
    let claim_id = config.next_claim_id;
    config.next_claim_id += 1;
    CLAIMS.save(
        deps.storage,
        claim_id,
        &ReimbursementClaim {
            claimant: claimant.clone(),
            amount,
            description,
            paid: false,
        },
    )?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "approve_claim")
        .add_attribute("claim_id", claim_id.to_string())
        .add_attribute("claimant", claimant)
        .add_attribute("amount", amount))
}

fn execute_claim(
    deps: DepsMut,
    info: MessageInfo,
    claim_id: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut claim = CLAIMS
        .may_load(deps.storage, claim_id)?
        .ok_or(ContractError::ClaimNotFound {})?;
    if claim.paid {
        return Err(ContractError::ClaimAlreadyPaid {});
    }
    if info.sender != claim.claimant {
        return Err(ContractError::WrongClaimant {});
    }
    if config.balance < claim.amount {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: claim.amount,
        });
    }

    config.balance = config
        .balance
        .checked_sub(claim.amount)
        .map_err(StdError::overflow)?;
    claim.paid = true;
    CONFIG.save(deps.storage, &config)?;
    CLAIMS.save(deps.storage, claim_id, &claim)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: claim.amount,
            }],
        })
        .add_attribute("action", "claim")
        .add_attribute("claim_id", claim_id.to_string())
        .add_attribute("claimant", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", claim.amount)
        .add_attribute("balance", config.balance))
}

fn execute_cancel_claim(
    deps: DepsMut,
    info: MessageInfo,
    claim_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let claim = CLAIMS
        .may_load(deps.storage, claim_id)?
        .ok_or(ContractError::ClaimNotFound {})?;
    if claim.paid {
        return Err(ContractError::ClaimAlreadyPaid {});
    }

    CLAIMS.remove(deps.storage, claim_id);

    Ok(Response::new()
        .add_attribute("action", "cancel_claim")
        .add_attribute("claim_id", claim_id.to_string()))
}

fn execute_withdraw_excess(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if config.balance < amount {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: amount,
        });
    }

    let recipient = deps.api.addr_validate(&recipient)?;
    config.balance = config
        .balance
        .checked_sub(amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount,
            }],
        })
        .add_attribute("action", "withdraw_excess")
        .add_attribute("recipient", recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", amount)
        .add_attribute("balance", config.balance))
}

fn require_single_coin<'a>(
    funds: &'a [Coin],
    expected_denom: &str,
) -> Result<&'a Coin, ContractError> {
    match funds {
        [] => Err(ContractError::MissingPayment {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount.is_zero() {
                return Err(ContractError::InvalidAmount {});
            }
            Ok(coin)
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Claim { claim_id } => to_binary(&query_claim(deps, claim_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        balance: config.balance,
        next_claim_id: config.next_claim_id,
    })
}

fn query_claim(deps: Deps, claim_id: u64) -> StdResult<ClaimResponse> {
    let claim = CLAIMS.load(deps.storage, claim_id)?;
    Ok(ClaimResponse {
        claim_id,
        claimant: claim.claimant.to_string(),
        amount: claim.amount,
        description: claim.description,
        paid: claim.paid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, CosmosMsg};

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: Some("owner".to_string()),
                denom: "uatom".to_string(),
            },
        )
        .unwrap();
    }

    fn fund_and_approve(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("funder", &coins(500, "uatom")),
            ExecuteMsg::Fund {},
        )
        .unwrap();
    }

    fn approve_claim(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::ApproveClaim {
                claimant: "alice".to_string(),
                amount: Uint128::new(125),
                description: Some("travel receipt".to_string()),
            },
        )
        .unwrap();
    }

    #[test]
    fn claimant_receives_approved_reimbursement() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        fund_and_approve(deps.as_mut());
        approve_claim(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Claim { claim_id: 1 },
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(125, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let claim: ClaimResponse = from_binary(
            &query(deps.as_ref(), mock_env(), QueryMsg::Claim { claim_id: 1 }).unwrap(),
        )
        .unwrap();
        assert!(claim.paid);

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(375));
    }

    #[test]
    fn rejects_unauthorized_approval_and_wrong_claimant() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        fund_and_approve(deps.as_mut());

        let unauthorized = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::ApproveClaim {
                claimant: "alice".to_string(),
                amount: Uint128::new(125),
                description: None,
            },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::Unauthorized {}));

        approve_claim(deps.as_mut());
        let wrong_claimant = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Claim { claim_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(wrong_claimant, ContractError::WrongClaimant {}));
    }

    #[test]
    fn claim_requires_pool_balance() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        approve_claim(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Claim { claim_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientBalance { .. }));
    }

    #[test]
    fn owner_cancels_unpaid_claim_and_withdraws_excess() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        fund_and_approve(deps.as_mut());
        approve_claim(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::CancelClaim { claim_id: 1 },
        )
        .unwrap();
        let missing =
            query(deps.as_ref(), mock_env(), QueryMsg::Claim { claim_id: 1 }).unwrap_err();
        assert!(missing.to_string().contains("not found"));

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::WithdrawExcess {
                recipient: "treasury".to_string(),
                amount: Uint128::new(200),
            },
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(200, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }
}
