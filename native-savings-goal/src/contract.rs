#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-savings-goal";
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
    if msg.target.is_zero() {
        return Err(ContractError::InvalidTarget {});
    }

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };
    let beneficiary = deps.api.addr_validate(&msg.beneficiary)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            beneficiary: beneficiary.clone(),
            denom: denom.clone(),
            target: msg.target,
            saved: Uint128::zero(),
            released: false,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("beneficiary", beneficiary)
        .add_attribute("denom", denom)
        .add_attribute("target", msg.target))
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
        ExecuteMsg::Release {} => execute_release(deps, info),
        ExecuteMsg::UpdateBeneficiary { beneficiary } => {
            execute_update_beneficiary(deps, info, beneficiary)
        }
    }
}

fn execute_fund(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.released {
        return Err(ContractError::AlreadyReleased {});
    }

    let coin = require_single_coin(&info.funds, &config.denom)?;
    config.saved = config
        .saved
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "fund")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("saved", config.saved)
        .add_attribute("target", config.target))
}

fn execute_release(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.released {
        return Err(ContractError::AlreadyReleased {});
    }
    if info.sender != config.beneficiary {
        return Err(ContractError::NotBeneficiary {});
    }
    if config.saved < config.target {
        return Err(ContractError::TargetNotReached {
            saved: config.saved,
            target: config.target,
        });
    }

    let payout = config.saved;
    config.saved = Uint128::zero();
    config.released = true;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: payout,
            }],
        })
        .add_attribute("action", "release")
        .add_attribute("beneficiary", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", payout))
}

fn execute_update_beneficiary(
    deps: DepsMut,
    info: MessageInfo,
    beneficiary: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if config.released {
        return Err(ContractError::AlreadyReleased {});
    }

    let beneficiary = deps.api.addr_validate(&beneficiary)?;
    config.beneficiary = beneficiary.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_beneficiary")
        .add_attribute("beneficiary", beneficiary))
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
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        beneficiary: config.beneficiary.to_string(),
        denom: config.denom,
        target: config.target,
        saved: config.saved,
        released: config.released,
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
                beneficiary: "beneficiary".to_string(),
                denom: "uatom".to_string(),
                target: Uint128::new(500),
            },
        )
        .unwrap();
    }

    #[test]
    fn funds_and_releases_after_target() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(250, "uatom")),
            ExecuteMsg::Fund {},
        )
        .unwrap();
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &coins(300, "uatom")),
            ExecuteMsg::Fund {},
        )
        .unwrap();

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("beneficiary", &[]),
            ExecuteMsg::Release {},
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "beneficiary");
                assert_eq!(amount, &coins(550, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert!(config.released);
        assert_eq!(config.saved, Uint128::zero());
    }

    #[test]
    fn rejects_release_before_target() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::Fund {},
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("beneficiary", &[]),
            ExecuteMsg::Release {},
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::TargetNotReached { .. }));
    }

    #[test]
    fn owner_can_update_beneficiary() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::UpdateBeneficiary {
                beneficiary: "new_beneficiary".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateBeneficiary {
                beneficiary: "new_beneficiary".to_string(),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.beneficiary, "new_beneficiary");
    }

    #[test]
    fn funding_requires_configured_native_denom() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(250, "uosmo")),
            ExecuteMsg::Fund {},
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::WrongDenom { .. }));
    }
}
