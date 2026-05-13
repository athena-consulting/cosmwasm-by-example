#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-sponsor-match";
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
    validate_match_bps(msg.match_bps)?;

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
            match_bps: msg.match_bps,
            match_pool: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("beneficiary", beneficiary)
        .add_attribute("denom", denom)
        .add_attribute("match_bps", msg.match_bps.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: cosmwasm_std::Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::FundMatchPool {} => execute_fund_match_pool(deps, info),
        ExecuteMsg::Donate {} => execute_donate(deps, info),
        ExecuteMsg::UpdateConfig {
            beneficiary,
            match_bps,
        } => execute_update_config(deps, info, beneficiary, match_bps),
    }
}

fn execute_fund_match_pool(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    config.match_pool = config
        .match_pool
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "fund_match_pool")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("match_pool", config.match_pool))
}

fn execute_donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    let match_amount = coin
        .amount
        .multiply_ratio(config.match_bps as u128, FULL_BPS as u128);
    if match_amount.is_zero() {
        return Err(ContractError::MatchTooSmall {});
    }
    if config.match_pool < match_amount {
        return Err(ContractError::InsufficientMatchPool {
            available: config.match_pool,
            required: match_amount,
        });
    }

    config.match_pool = config
        .match_pool
        .checked_sub(match_amount)
        .map_err(StdError::overflow)?;
    let total = coin
        .amount
        .checked_add(match_amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: config.beneficiary.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: total,
            }],
        })
        .add_attribute("action", "donate")
        .add_attribute("donor", info.sender)
        .add_attribute("beneficiary", config.beneficiary)
        .add_attribute("denom", config.denom)
        .add_attribute("donation", coin.amount)
        .add_attribute("match_amount", match_amount)
        .add_attribute("total_sent", total)
        .add_attribute("match_pool", config.match_pool))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    beneficiary: Option<String>,
    match_bps: Option<u16>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(beneficiary) = beneficiary {
        config.beneficiary = deps.api.addr_validate(&beneficiary)?;
    }
    if let Some(match_bps) = match_bps {
        validate_match_bps(match_bps)?;
        config.match_bps = match_bps;
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("beneficiary", config.beneficiary)
        .add_attribute("match_bps", config.match_bps.to_string()))
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

fn validate_match_bps(match_bps: u16) -> Result<(), ContractError> {
    if match_bps == 0 || match_bps > FULL_BPS {
        return Err(ContractError::InvalidMatchBps {});
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
        beneficiary: config.beneficiary.to_string(),
        denom: config.denom,
        match_bps: config.match_bps,
        match_pool: config.match_pool,
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
                beneficiary: "beneficiary".to_string(),
                denom: "uatom".to_string(),
                match_bps: 5000,
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_validates_denom_and_match_bps() {
        let mut deps = mock_dependencies();

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: None,
                beneficiary: "beneficiary".to_string(),
                denom: " ".to_string(),
                match_bps: 5000,
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
                beneficiary: "beneficiary".to_string(),
                denom: "uatom".to_string(),
                match_bps: 0,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidMatchBps {}));
    }

    #[test]
    fn fund_match_pool_validates_funds_and_updates_pool() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[]),
            ExecuteMsg::FundMatchPool {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MissingPayment {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[coin(100, "uosmo")]),
            ExecuteMsg::FundMatchPool {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::WrongDenom { .. }));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[coin(100, "uatom")]),
            ExecuteMsg::FundMatchPool {},
        )
        .unwrap();

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.match_pool, Uint128::new(100));
    }

    #[test]
    fn donate_requires_match_pool_and_sends_donation_plus_match() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("donor", &[coin(20, "uatom")]),
            ExecuteMsg::Donate {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientMatchPool { .. }));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[coin(100, "uatom")]),
            ExecuteMsg::FundMatchPool {},
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("donor", &[coin(20, "uatom")]),
            ExecuteMsg::Donate {},
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "beneficiary");
                assert_eq!(amount, &coins(30, "uatom"));
            }
            _ => panic!("expected bank send"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.match_pool, Uint128::new(90));
    }

    #[test]
    fn tiny_donation_with_zero_match_is_rejected() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("sponsor", &[coin(100, "uatom")]),
            ExecuteMsg::FundMatchPool {},
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("donor", &[coin(1, "uatom")]),
            ExecuteMsg::Donate {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MatchTooSmall {}));
    }

    #[test]
    fn only_owner_can_update_config() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::UpdateConfig {
                beneficiary: Some("new_beneficiary".to_string()),
                match_bps: Some(2500),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                beneficiary: Some("new_beneficiary".to_string()),
                match_bps: Some(2500),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.beneficiary, "new_beneficiary");
        assert_eq!(config.match_bps, 2500);
    }
}
