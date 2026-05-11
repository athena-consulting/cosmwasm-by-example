#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, LastClaimResponse, NextClaimResponse, QueryMsg,
};
use crate::state::{Config, CONFIG, LAST_CLAIM};

const CONTRACT_NAME: &str = "crates.io:rate-limited-faucet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_denom(&msg.denom)?;
    validate_claim_amount(msg.claim_amount)?;

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender.clone(),
    };

    let config = Config {
        owner,
        denom: msg.denom,
        claim_amount: msg.claim_amount,
        cooldown_seconds: msg.cooldown_seconds,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", config.owner)
        .add_attribute("denom", config.denom)
        .add_attribute("claim_amount", config.claim_amount)
        .add_attribute("cooldown_seconds", config.cooldown_seconds.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::Fund {} => execute_fund(info),
        ExecuteMsg::UpdateConfig {
            owner,
            claim_amount,
            cooldown_seconds,
        } => execute_update_config(deps, info, owner, claim_amount, cooldown_seconds),
    }
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    if let Some(last_claim_at) = LAST_CLAIM.may_load(deps.storage, &info.sender)? {
        let next_claim_at = last_claim_at + config.cooldown_seconds;
        if now < next_claim_at {
            return Err(ContractError::CooldownNotExpired { next_claim_at });
        }
    }

    LAST_CLAIM.save(deps.storage, &info.sender, &now)?;

    let payout = coin(config.claim_amount.u128(), config.denom.clone());
    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![payout],
        })
        .add_attribute("action", "claim")
        .add_attribute("recipient", info.sender)
        .add_attribute("amount", config.claim_amount)
        .add_attribute("denom", config.denom)
        .add_attribute("claimed_at", now.to_string()))
}

pub fn execute_fund(info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_attribute("action", "fund")
        .add_attribute("sender", info.sender)
        .add_attribute("fund_count", info.funds.len().to_string()))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    claim_amount: Option<cosmwasm_std::Uint128>,
    cooldown_seconds: Option<u64>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }

        if let Some(claim_amount) = claim_amount {
            validate_claim_amount(claim_amount)?;
            config.claim_amount = claim_amount;
        }

        if let Some(cooldown_seconds) = cooldown_seconds {
            config.cooldown_seconds = cooldown_seconds;
        }

        Ok(config)
    })?;

    let config = CONFIG.load(deps.storage)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("owner", config.owner)
        .add_attribute("claim_amount", config.claim_amount)
        .add_attribute("cooldown_seconds", config.cooldown_seconds.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::LastClaim { address } => to_json_binary(&query_last_claim(deps, address)?),
        QueryMsg::NextClaim { address } => to_json_binary(&query_next_claim(deps, env, address)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        claim_amount: config.claim_amount,
        cooldown_seconds: config.cooldown_seconds,
    })
}

pub fn query_last_claim(deps: Deps, address: String) -> StdResult<LastClaimResponse> {
    let address = deps.api.addr_validate(&address)?;
    let last_claim_at = LAST_CLAIM.may_load(deps.storage, &address)?;
    Ok(LastClaimResponse {
        address: address.to_string(),
        last_claim_at,
    })
}

pub fn query_next_claim(deps: Deps, env: Env, address: String) -> StdResult<NextClaimResponse> {
    let config = CONFIG.load(deps.storage)?;
    let address = deps.api.addr_validate(&address)?;
    let next_claim_at = LAST_CLAIM
        .may_load(deps.storage, &address)?
        .map(|last_claim_at| last_claim_at + config.cooldown_seconds)
        .unwrap_or(0);

    Ok(NextClaimResponse {
        address: address.to_string(),
        next_claim_at,
        can_claim: env.block.time.seconds() >= next_claim_at,
    })
}

fn validate_claim_amount(amount: cosmwasm_std::Uint128) -> Result<(), ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidClaimAmount {});
    }
    Ok(())
}

fn validate_denom(denom: &str) -> Result<(), ContractError> {
    if denom.trim().is_empty() {
        return Err(ContractError::InvalidDenom {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, BankMsg, CosmosMsg, Timestamp, Uint128};

    fn instantiate_faucet(deps: DepsMut) {
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {
            owner: None,
            denom: "utest".to_string(),
            claim_amount: Uint128::new(25),
            cooldown_seconds: 60,
        };
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn instantiate_and_query_config() {
        let mut deps = mock_dependencies();
        instantiate_faucet(deps.as_mut());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.owner, "owner");
        assert_eq!(config.denom, "utest");
        assert_eq!(config.claim_amount, Uint128::new(25));
        assert_eq!(config.cooldown_seconds, 60);
    }

    #[test]
    fn claim_sends_tokens_and_records_timestamp() {
        let mut deps = mock_dependencies();
        instantiate_faucet(deps.as_mut());

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let info = mock_info("claimer", &[]);
        let res = execute_claim(deps.as_mut(), env.clone(), info).unwrap();

        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "claimer");
                assert_eq!(amount, &coins(25, "utest"));
            }
            _ => panic!("expected bank send"),
        }

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::LastClaim {
                address: "claimer".to_string(),
            },
        )
        .unwrap();
        let last_claim: LastClaimResponse = from_json(&res).unwrap();
        assert_eq!(last_claim.last_claim_at, Some(100));
    }

    #[test]
    fn claim_is_rate_limited_per_address() {
        let mut deps = mock_dependencies();
        instantiate_faucet(deps.as_mut());

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);
        let info = mock_info("claimer", &[]);
        execute_claim(deps.as_mut(), env.clone(), info.clone()).unwrap();

        env.block.time = Timestamp::from_seconds(120);
        let err = execute_claim(deps.as_mut(), env.clone(), info.clone()).unwrap_err();
        assert_eq!(
            err,
            ContractError::CooldownNotExpired { next_claim_at: 160 }
        );

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::NextClaim {
                address: "claimer".to_string(),
            },
        )
        .unwrap();
        let next_claim: NextClaimResponse = from_json(&res).unwrap();
        assert_eq!(next_claim.next_claim_at, 160);
        assert!(!next_claim.can_claim);

        env.block.time = Timestamp::from_seconds(160);
        execute_claim(deps.as_mut(), env, info).unwrap();
    }

    #[test]
    fn owner_can_update_config() {
        let mut deps = mock_dependencies();
        instantiate_faucet(deps.as_mut());

        let err = execute_update_config(
            deps.as_mut(),
            mock_info("not-owner", &[]),
            None,
            Some(Uint128::new(10)),
            None,
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute_update_config(
            deps.as_mut(),
            mock_info("owner", &[]),
            Some("new-owner".to_string()),
            Some(Uint128::new(10)),
            Some(30),
        )
        .unwrap();

        let config = query_config(deps.as_ref()).unwrap();
        assert_eq!(config.owner, "new-owner");
        assert_eq!(config.claim_amount, Uint128::new(10));
        assert_eq!(config.cooldown_seconds, 30);
    }

    #[test]
    fn rejects_empty_denom_and_zero_claim_amount() {
        let mut deps = mock_dependencies();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {
            owner: None,
            denom: " ".to_string(),
            claim_amount: Uint128::new(25),
            cooldown_seconds: 60,
        };
        let err = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidDenom {});

        let msg = InstantiateMsg {
            owner: None,
            denom: "utest".to_string(),
            claim_amount: Uint128::zero(),
            cooldown_seconds: 60,
        };
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidClaimAmount {});
    }
}
