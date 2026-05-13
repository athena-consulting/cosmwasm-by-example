#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    BountyResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, NextIdResponse, QueryMsg,
};
use crate::state::{Bounty, Config, BOUNTIES, CONFIG, NEXT_ID};

const CONTRACT_NAME: &str = "crates.io:native-bounty-board";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: cosmwasm_std::Env,
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
        },
    )?;
    NEXT_ID.save(deps.storage, &1)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: cosmwasm_std::Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateBounty { title } => execute_create_bounty(deps, info, title),
        ExecuteMsg::Award { id, recipient } => execute_award(deps, info, id, recipient),
        ExecuteMsg::Cancel { id } => execute_cancel(deps, info, id),
    }
}

fn execute_create_bounty(
    deps: DepsMut,
    info: MessageInfo,
    title: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ContractError::EmptyTitle {});
    }
    let coin = require_single_coin(&info.funds, &config.denom)?;
    let id = NEXT_ID.load(deps.storage)?;

    BOUNTIES.save(
        deps.storage,
        id,
        &Bounty {
            sponsor: info.sender.clone(),
            title: title.clone(),
            amount: coin.amount,
            settled: false,
        },
    )?;
    NEXT_ID.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_bounty")
        .add_attribute("id", id.to_string())
        .add_attribute("sponsor", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("title", title))
}

fn execute_award(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    recipient: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut bounty = BOUNTIES
        .may_load(deps.storage, id)?
        .ok_or(ContractError::BountyNotFound {})?;
    if info.sender != bounty.sponsor {
        return Err(ContractError::Unauthorized {});
    }
    if bounty.settled {
        return Err(ContractError::BountySettled {});
    }
    let recipient = deps.api.addr_validate(&recipient)?;

    bounty.settled = true;
    BOUNTIES.save(deps.storage, id, &bounty)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: bounty.amount,
            }],
        })
        .add_attribute("action", "award")
        .add_attribute("id", id.to_string())
        .add_attribute("sponsor", bounty.sponsor)
        .add_attribute("recipient", recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", bounty.amount))
}

fn execute_cancel(deps: DepsMut, info: MessageInfo, id: u64) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut bounty = BOUNTIES
        .may_load(deps.storage, id)?
        .ok_or(ContractError::BountyNotFound {})?;
    if info.sender != bounty.sponsor {
        return Err(ContractError::Unauthorized {});
    }
    if bounty.settled {
        return Err(ContractError::BountySettled {});
    }

    bounty.settled = true;
    BOUNTIES.save(deps.storage, id, &bounty)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: bounty.sponsor.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: bounty.amount,
            }],
        })
        .add_attribute("action", "cancel")
        .add_attribute("id", id.to_string())
        .add_attribute("sponsor", bounty.sponsor)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", bounty.amount))
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
pub fn query(deps: Deps, _env: cosmwasm_std::Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Bounty { id } => to_binary(&query_bounty(deps, id)?),
        QueryMsg::NextId {} => to_binary(&query_next_id(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
    })
}

fn query_bounty(deps: Deps, id: u64) -> StdResult<BountyResponse> {
    let bounty = BOUNTIES.load(deps.storage, id)?;
    Ok(BountyResponse {
        id,
        sponsor: bounty.sponsor.to_string(),
        title: bounty.title,
        amount: bounty.amount,
        settled: bounty.settled,
    })
}

fn query_next_id(deps: Deps) -> StdResult<NextIdResponse> {
    Ok(NextIdResponse {
        next_id: NEXT_ID.load(deps.storage)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, CosmosMsg};

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

    fn create_bounty(deps: DepsMut) {
        execute(
            deps,
            mock_env(),
            mock_info("sponsor", &coins(75, "uatom")),
            ExecuteMsg::CreateBounty {
                title: "Fix wallet bug".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn create_bounty_records_funded_task() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        create_bounty(deps.as_mut());

        let response: BountyResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Bounty { id: 1 }).unwrap())
                .unwrap();
        assert_eq!(response.sponsor, "sponsor");
        assert_eq!(response.title, "Fix wallet bug");
        assert_eq!(response.amount, coin(75, "uatom").amount);
        assert!(!response.settled);

        let next: NextIdResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::NextId {}).unwrap()).unwrap();
        assert_eq!(next.next_id, 2);
    }

    #[test]
    fn award_sends_bounty_to_recipient() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_bounty(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[]),
            ExecuteMsg::Award {
                id: 1,
                recipient: "solver".to_string(),
            },
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "solver");
                assert_eq!(amount, &coins(75, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let response: BountyResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Bounty { id: 1 }).unwrap())
                .unwrap();
        assert!(response.settled);
    }

    #[test]
    fn cancel_refunds_sponsor() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_bounty(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[]),
            ExecuteMsg::Cancel { id: 1 },
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "sponsor");
                assert_eq!(amount, &coins(75, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }

    #[test]
    fn rejects_invalid_creation_and_unauthorized_settlement() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[]),
            ExecuteMsg::CreateBounty {
                title: "Fix wallet bug".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MissingPayment {}));

        create_bounty(deps.as_mut());
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::Award {
                id: 1,
                recipient: "solver".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }
}
