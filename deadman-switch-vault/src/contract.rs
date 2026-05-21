#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StatusResponse};
use crate::state::{Config, CONFIG, LAST_HEARTBEAT};

const CONTRACT_NAME: &str = "crates.io:deadman-switch-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_denom(&msg.denom)?;
    validate_window(msg.heartbeat_window)?;

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender.clone(),
    };
    let beneficiary = deps.api.addr_validate(&msg.beneficiary)?;

    let config = Config {
        owner,
        beneficiary,
        denom: msg.denom,
        heartbeat_window: msg.heartbeat_window,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    LAST_HEARTBEAT.save(deps.storage, &env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", config.owner)
        .add_attribute("beneficiary", config.beneficiary)
        .add_attribute("denom", config.denom)
        .add_attribute("heartbeat_window", config.heartbeat_window.to_string())
        .add_attribute("last_heartbeat", env.block.height.to_string()))
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
        ExecuteMsg::Heartbeat {} => execute_heartbeat(deps, env, info),
        ExecuteMsg::UpdateConfig {
            owner,
            beneficiary,
            heartbeat_window,
        } => execute_update_config(deps, info, owner, beneficiary, heartbeat_window),
        ExecuteMsg::Claim { recipient, amount } => {
            execute_claim(deps, env, info, recipient, amount)
        }
    }
}

fn execute_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let deposited = amount_sent(&info.funds, &config.denom);
    if deposited.is_zero() {
        return Err(ContractError::MissingDeposit {
            denom: config.denom,
        });
    }

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender)
        .add_attribute("amount", deposited)
        .add_attribute("denom", config.denom))
}

fn execute_heartbeat(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_owner(&info, &config)?;
    LAST_HEARTBEAT.save(deps.storage, &env.block.height)?;

    Ok(Response::new()
        .add_attribute("action", "heartbeat")
        .add_attribute("owner", info.sender)
        .add_attribute("last_heartbeat", env.block.height.to_string()))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    beneficiary: Option<String>,
    heartbeat_window: Option<u64>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        assert_owner(&info, &config)?;

        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(beneficiary) = beneficiary {
            config.beneficiary = deps.api.addr_validate(&beneficiary)?;
        }
        if let Some(heartbeat_window) = heartbeat_window {
            validate_window(heartbeat_window)?;
            config.heartbeat_window = heartbeat_window;
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_beneficiary(&info, &config)?;
    if !is_expired(deps.as_ref(), &env, &config)? {
        return Err(ContractError::NotExpired {});
    }

    let recipient = match recipient {
        Some(recipient) => deps.api.addr_validate(&recipient)?,
        None => config.beneficiary.clone(),
    };
    let balance = deps
        .querier
        .query_balance(env.contract.address, config.denom.clone())?
        .amount;
    let amount = amount.unwrap_or(balance);
    if amount.is_zero() {
        return Err(ContractError::ZeroAmount {});
    }
    if amount > balance {
        return Err(ContractError::InsufficientBalance {
            available: balance,
            amount,
        });
    }

    let msg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin::new(amount.u128(), config.denom.clone())],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "claim")
        .add_attribute("beneficiary", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("amount", amount)
        .add_attribute("denom", config.denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Status {} => to_json_binary(&query_status(deps, env)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        beneficiary: config.beneficiary.to_string(),
        denom: config.denom,
        heartbeat_window: config.heartbeat_window,
    })
}

fn query_status(deps: Deps, env: Env) -> StdResult<StatusResponse> {
    let config = CONFIG.load(deps.storage)?;
    let last_heartbeat = LAST_HEARTBEAT.load(deps.storage)?;
    let expires_at = last_heartbeat + config.heartbeat_window;
    Ok(StatusResponse {
        owner: config.owner.to_string(),
        beneficiary: config.beneficiary.to_string(),
        denom: config.denom,
        heartbeat_window: config.heartbeat_window,
        last_heartbeat,
        expires_at,
        expired: env.block.height > expires_at,
    })
}

fn is_expired(deps: Deps, env: &Env, config: &Config) -> StdResult<bool> {
    let last_heartbeat = LAST_HEARTBEAT.load(deps.storage)?;
    Ok(env.block.height > last_heartbeat + config.heartbeat_window)
}

fn amount_sent(funds: &[Coin], denom: &str) -> Uint128 {
    funds
        .iter()
        .filter(|coin| coin.denom == denom)
        .map(|coin| coin.amount)
        .sum()
}

fn assert_owner(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn assert_beneficiary(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.sender != config.beneficiary {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn validate_denom(denom: &str) -> Result<(), ContractError> {
    if denom.trim().is_empty() {
        return Err(ContractError::InvalidDenom {});
    }
    Ok(())
}

fn validate_window(heartbeat_window: u64) -> Result<(), ContractError> {
    if heartbeat_window == 0 {
        return Err(ContractError::InvalidHeartbeatWindow {});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_json, Addr, CosmosMsg};

    fn instantiate_default(deps: DepsMut, height: u64) {
        let mut env = mock_env();
        env.block.height = height;
        let msg = InstantiateMsg {
            owner: None,
            beneficiary: "beneficiary".to_string(),
            denom: "utest".to_string(),
            heartbeat_window: 10,
        };
        instantiate(deps, env, mock_info("owner", &[]), msg).unwrap();
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut(), 20);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.owner, "owner");
        assert_eq!(config.beneficiary, "beneficiary");
        assert_eq!(config.denom, "utest");
        assert_eq!(config.heartbeat_window, 10);
    }

    #[test]
    fn deposit_requires_configured_denom() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut(), 20);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("depositor", &coins(10, "wrong")),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        match err {
            ContractError::MissingDeposit { denom } => assert_eq!(denom, "utest"),
            _ => panic!("unexpected error: {err}"),
        }

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("depositor", &coins(10, "utest")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "deposit");
    }

    #[test]
    fn heartbeat_extends_liveness() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut(), 20);

        let mut env = mock_env();
        env.block.height = 25;
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::Heartbeat {},
        )
        .unwrap();

        env.block.height = 34;
        let res = query(deps.as_ref(), env.clone(), QueryMsg::Status {}).unwrap();
        let status: StatusResponse = from_json(&res).unwrap();
        assert_eq!(status.last_heartbeat, 25);
        assert_eq!(status.expires_at, 35);
        assert!(!status.expired);

        env.block.height = 36;
        let res = query(deps.as_ref(), env, QueryMsg::Status {}).unwrap();
        let status: StatusResponse = from_json(&res).unwrap();
        assert!(status.expired);
    }

    #[test]
    fn only_owner_updates_config() {
        let mut deps = mock_dependencies();
        instantiate_default(deps.as_mut(), 20);

        let msg = ExecuteMsg::UpdateConfig {
            owner: None,
            beneficiary: Some("new_beneficiary".to_string()),
            heartbeat_window: Some(25),
        };
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("other", &[]),
            msg.clone(),
        )
        .unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            _ => panic!("unexpected error: {err}"),
        }

        execute(deps.as_mut(), mock_env(), mock_info("owner", &[]), msg).unwrap();
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        assert_eq!(config.beneficiary, "new_beneficiary");
        assert_eq!(config.heartbeat_window, 25);
    }

    #[test]
    fn beneficiary_can_claim_after_expiration() {
        let mut deps = mock_dependencies();
        deps.querier
            .update_balance(Addr::unchecked("cosmos2contract"), vec![coin(175, "utest")]);
        instantiate_default(deps.as_mut(), 20);

        let mut env = mock_env();
        env.contract.address = Addr::unchecked("cosmos2contract");
        env.block.height = 30;
        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("beneficiary", &[]),
            ExecuteMsg::Claim {
                recipient: None,
                amount: None,
            },
        )
        .unwrap_err();
        match err {
            ContractError::NotExpired {} => {}
            _ => panic!("unexpected error: {err}"),
        }

        env.block.height = 31;
        let res = execute(
            deps.as_mut(),
            env,
            mock_info("beneficiary", &[]),
            ExecuteMsg::Claim {
                recipient: Some("recipient".to_string()),
                amount: Some(Uint128::new(125)),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "recipient");
                assert_eq!(amount, &coins(125, "utest"));
            }
            _ => panic!("unexpected message"),
        }
    }
}
