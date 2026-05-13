#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;
use std::collections::BTreeSet;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ShareInput, ShareResponse};
use crate::state::{Config, RoyaltyShare, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-royalty-router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const FULL_BPS: u16 = 10_000;

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
    let shares = validate_shares(deps.as_ref(), msg.shares)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            denom: denom.clone(),
            shares,
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
    _env: cosmwasm_std::Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RoutePayment {} => execute_route_payment(deps, info),
        ExecuteMsg::UpdateShares { shares } => execute_update_shares(deps, info, shares),
        ExecuteMsg::UpdateOwner { owner } => execute_update_owner(deps, info, owner),
    }
}

fn execute_route_payment(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    let mut messages = Vec::with_capacity(config.shares.len());
    let mut distributed = Uint128::zero();
    let mut amounts = Vec::with_capacity(config.shares.len());

    for share in &config.shares {
        let amount = coin
            .amount
            .multiply_ratio(share.bps as u128, FULL_BPS as u128);
        if amount.is_zero() {
            return Err(ContractError::PaymentTooSmall {});
        }
        distributed = distributed
            .checked_add(amount)
            .map_err(StdError::overflow)?;
        amounts.push(amount);
    }

    let remainder = coin
        .amount
        .checked_sub(distributed)
        .map_err(StdError::overflow)?;
    amounts[0] = amounts[0]
        .checked_add(remainder)
        .map_err(StdError::overflow)?;

    for (share, amount) in config.shares.iter().zip(amounts) {
        messages.push(BankMsg::Send {
            to_address: share.recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount,
            }],
        });
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "route_payment")
        .add_attribute("payer", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("recipient_count", config.shares.len().to_string()))
}

fn execute_update_shares(
    deps: DepsMut,
    info: MessageInfo,
    shares: Vec<ShareInput>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info)?;

    let shares = validate_shares(deps.as_ref(), shares)?;
    let share_count = shares.len();
    config.shares = shares;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_shares")
        .add_attribute("share_count", share_count.to_string()))
}

fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    assert_owner(&config, &info)?;

    let new_owner = deps.api.addr_validate(&owner)?;
    if new_owner == config.owner {
        return Err(ContractError::SameOwner {});
    }

    let previous_owner = config.owner;
    config.owner = new_owner.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_owner")
        .add_attribute("previous_owner", previous_owner)
        .add_attribute("new_owner", new_owner))
}

fn validate_shares(
    deps: Deps,
    shares: Vec<ShareInput>,
) -> Result<Vec<RoyaltyShare>, ContractError> {
    if shares.is_empty() {
        return Err(ContractError::EmptyShares {});
    }

    let mut seen = BTreeSet::new();
    let mut total: u32 = 0;
    let mut validated = Vec::with_capacity(shares.len());

    for share in shares {
        if share.bps == 0 {
            return Err(ContractError::InvalidShareBps {});
        }
        let recipient = deps.api.addr_validate(&share.recipient)?;
        if !seen.insert(recipient.to_string()) {
            return Err(ContractError::DuplicateRecipient {});
        }
        total += share.bps as u32;
        validated.push(RoyaltyShare {
            recipient,
            bps: share.bps,
        });
    }

    if total != FULL_BPS as u32 {
        return Err(ContractError::InvalidShareTotal {});
    }

    Ok(validated)
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
                return Err(ContractError::InvalidPaymentAmount {});
            }
            Ok(coin)
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

fn assert_owner(config: &Config, info: &MessageInfo) -> Result<(), ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: cosmwasm_std::Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        shares: config
            .shares
            .into_iter()
            .map(|share| ShareResponse {
                recipient: share.recipient.to_string(),
                bps: share.bps,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, CosmosMsg};

    fn shares() -> Vec<ShareInput> {
        vec![
            ShareInput {
                recipient: "artist".to_string(),
                bps: 7000,
            },
            ShareInput {
                recipient: "producer".to_string(),
                bps: 3000,
            },
        ]
    }

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: Some("owner".to_string()),
                denom: "uatom".to_string(),
                shares: shares(),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_validates_denom_and_shares() {
        let mut deps = mock_dependencies();

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                denom: " ".to_string(),
                shares: shares(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EmptyDenom {}));

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                denom: "uatom".to_string(),
                shares: vec![],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EmptyShares {}));

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                denom: "uatom".to_string(),
                shares: vec![ShareInput {
                    recipient: "artist".to_string(),
                    bps: 9999,
                }],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidShareTotal {}));
    }

    #[test]
    fn duplicate_or_zero_share_is_rejected() {
        let mut deps = mock_dependencies();

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                denom: "uatom".to_string(),
                shares: vec![
                    ShareInput {
                        recipient: "artist".to_string(),
                        bps: 5000,
                    },
                    ShareInput {
                        recipient: "artist".to_string(),
                        bps: 5000,
                    },
                ],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::DuplicateRecipient {}));

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                denom: "uatom".to_string(),
                shares: vec![ShareInput {
                    recipient: "artist".to_string(),
                    bps: 0,
                }],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidShareBps {}));
    }

    #[test]
    fn route_payment_validates_funds_and_splits_with_remainder() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[]),
            ExecuteMsg::RoutePayment {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MissingPayment {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[coin(10, "uosmo")]),
            ExecuteMsg::RoutePayment {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::WrongDenom { .. }));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[coin(101, "uatom")]),
            ExecuteMsg::RoutePayment {},
        )
        .unwrap();
        assert_eq!(res.messages.len(), 2);

        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "artist");
                assert_eq!(amount, &coins(71, "uatom"));
            }
            _ => panic!("expected bank send"),
        }
        match &res.messages[1].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "producer");
                assert_eq!(amount, &coins(30, "uatom"));
            }
            _ => panic!("expected bank send"),
        }
    }

    #[test]
    fn small_payment_is_rejected_when_a_share_would_be_zero() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("payer", &[coin(1, "uatom")]),
            ExecuteMsg::RoutePayment {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::PaymentTooSmall {}));
    }

    #[test]
    fn only_owner_can_update_shares_and_owner() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::UpdateShares {
                shares: vec![ShareInput {
                    recipient: "artist".to_string(),
                    bps: FULL_BPS,
                }],
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateShares {
                shares: vec![ShareInput {
                    recipient: "artist".to_string(),
                    bps: FULL_BPS,
                }],
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.shares.len(), 1);
        assert_eq!(config.shares[0].bps, FULL_BPS);

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateOwner {
                owner: "new_owner".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateShares { shares: shares() },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }
}
