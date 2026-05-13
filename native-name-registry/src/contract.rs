#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, NameResponse, QueryMsg};
use crate::state::{Config, NameRecord, CONFIG, NAMES};

const CONTRACT_NAME: &str = "crates.io:native-name-registry";
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
    if msg.registration_fee.is_zero() {
        return Err(ContractError::InvalidFee {});
    }
    if msg.registration_seconds == 0 {
        return Err(ContractError::InvalidRegistrationPeriod {});
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
            registration_fee: msg.registration_fee,
            registration_seconds: msg.registration_seconds,
            balance: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("registration_fee", msg.registration_fee)
        .add_attribute("registration_seconds", msg.registration_seconds.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register { name, value } => execute_register(deps, env, info, name, value),
        ExecuteMsg::Renew { name } => execute_renew(deps, env, info, name),
        ExecuteMsg::UpdateValue { name, value } => {
            execute_update_value(deps, env, info, name, value)
        }
        ExecuteMsg::Release { name } => execute_release(deps, env, info, name),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
    }
}

fn execute_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    value: String,
) -> Result<Response, ContractError> {
    let name = normalize_name(&name)?;
    let value = validate_value(value)?;
    let mut config = CONFIG.load(deps.storage)?;
    require_payment(&info.funds, &config.denom, config.registration_fee)?;

    if let Some(record) = NAMES.may_load(deps.storage, &name)? {
        if !is_expired(&record, &env) {
            return Err(ContractError::NameUnavailable {});
        }
    }

    let expires_at = env
        .block
        .time
        .seconds()
        .checked_add(config.registration_seconds)
        .ok_or_else(|| StdError::generic_err("expiry overflow"))?;
    NAMES.save(
        deps.storage,
        &name,
        &NameRecord {
            owner: info.sender.clone(),
            value,
            expires_at,
        },
    )?;
    config.balance = config
        .balance
        .checked_add(config.registration_fee)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "register")
        .add_attribute("name", name)
        .add_attribute("owner", info.sender)
        .add_attribute("expires_at", expires_at.to_string())
        .add_attribute("balance", config.balance))
}

fn execute_renew(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    let name = normalize_name(&name)?;
    let mut config = CONFIG.load(deps.storage)?;
    require_payment(&info.funds, &config.denom, config.registration_fee)?;

    let mut record = NAMES.load(deps.storage, &name)?;
    if record.owner != info.sender {
        return Err(ContractError::NotNameOwner {});
    }
    let base = record.expires_at.max(env.block.time.seconds());
    record.expires_at = base
        .checked_add(config.registration_seconds)
        .ok_or_else(|| StdError::generic_err("expiry overflow"))?;
    NAMES.save(deps.storage, &name, &record)?;
    config.balance = config
        .balance
        .checked_add(config.registration_fee)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "renew")
        .add_attribute("name", name)
        .add_attribute("owner", info.sender)
        .add_attribute("expires_at", record.expires_at.to_string())
        .add_attribute("balance", config.balance))
}

fn execute_update_value(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    value: String,
) -> Result<Response, ContractError> {
    let name = normalize_name(&name)?;
    let value = validate_value(value)?;
    let mut record = NAMES.load(deps.storage, &name)?;
    if record.owner != info.sender {
        return Err(ContractError::NotNameOwner {});
    }
    if is_expired(&record, &env) {
        return Err(ContractError::NameExpired {});
    }

    record.value = value;
    NAMES.save(deps.storage, &name, &record)?;

    Ok(Response::new()
        .add_attribute("action", "update_value")
        .add_attribute("name", name)
        .add_attribute("owner", info.sender))
}

fn execute_release(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    let name = normalize_name(&name)?;
    let record = NAMES.load(deps.storage, &name)?;
    if record.owner != info.sender {
        return Err(ContractError::NotNameOwner {});
    }
    if is_expired(&record, &env) {
        return Err(ContractError::NameExpired {});
    }

    NAMES.remove(deps.storage, &name);
    Ok(Response::new()
        .add_attribute("action", "release")
        .add_attribute("name", name)
        .add_attribute("owner", info.sender))
}

fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InvalidWithdrawAmount {});
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
        .add_attribute("action", "withdraw")
        .add_attribute("recipient", recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", amount)
        .add_attribute("balance", config.balance))
}

fn normalize_name(name: &str) -> Result<String, ContractError> {
    let normalized = name.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(ContractError::EmptyName {});
    }
    Ok(normalized)
}

fn validate_value(value: String) -> Result<String, ContractError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(ContractError::EmptyValue {});
    }
    Ok(value)
}

fn is_expired(record: &NameRecord, env: &Env) -> bool {
    env.block.time.seconds() >= record.expires_at
}

fn require_payment(
    funds: &[Coin],
    expected_denom: &str,
    required: Uint128,
) -> Result<(), ContractError> {
    match funds {
        [] => Err(ContractError::MissingPayment {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount < required {
                return Err(ContractError::Underpaid {
                    paid: coin.amount,
                    required,
                });
            }
            Ok(())
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Name { name } => to_binary(&query_name(deps, env, name)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        registration_fee: config.registration_fee,
        registration_seconds: config.registration_seconds,
        balance: config.balance,
    })
}

fn query_name(deps: Deps, env: Env, name: String) -> StdResult<NameResponse> {
    let name = normalize_name(&name).map_err(|err| StdError::generic_err(err.to_string()))?;
    let record = NAMES.may_load(deps.storage, &name)?;
    Ok(match record {
        Some(record) => {
            let expired = is_expired(&record, &env);
            NameResponse {
                name,
                owner: Some(record.owner.to_string()),
                value: Some(record.value),
                expires_at: Some(record.expires_at),
                is_expired: expired,
            }
        }
        None => NameResponse {
            name,
            owner: None,
            value: None,
            expires_at: None,
            is_expired: false,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, CosmosMsg, Timestamp};

    fn env_at(seconds: u64) -> Env {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(seconds);
        env
    }

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            env_at(100),
            mock_info("creator", &[]),
            InstantiateMsg {
                owner: Some("owner".to_string()),
                denom: "uatom".to_string(),
                registration_fee: Uint128::new(10),
                registration_seconds: 100,
            },
        )
        .unwrap();
    }

    #[test]
    fn registers_and_updates_name() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(10, "uatom")),
            ExecuteMsg::Register {
                name: "Alice".to_string(),
                value: "addr1".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env_at(120),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateValue {
                name: "alice".to_string(),
                value: "addr2".to_string(),
            },
        )
        .unwrap();

        let record: NameResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(120),
                QueryMsg::Name {
                    name: "ALICE".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(record.owner, Some("alice".to_string()));
        assert_eq!(record.value, Some("addr2".to_string()));
        assert_eq!(record.expires_at, Some(200));
        assert!(!record.is_expired);
    }

    #[test]
    fn blocks_duplicate_until_expiry_then_allows_reuse() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(10, "uatom")),
            ExecuteMsg::Register {
                name: "handle".to_string(),
                value: "alice-value".to_string(),
            },
        )
        .unwrap();
        let duplicate = execute(
            deps.as_mut(),
            env_at(150),
            mock_info("bob", &coins(10, "uatom")),
            ExecuteMsg::Register {
                name: "handle".to_string(),
                value: "bob-value".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(duplicate, ContractError::NameUnavailable {}));

        execute(
            deps.as_mut(),
            env_at(200),
            mock_info("bob", &coins(10, "uatom")),
            ExecuteMsg::Register {
                name: "handle".to_string(),
                value: "bob-value".to_string(),
            },
        )
        .unwrap();
        let record: NameResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(200),
                QueryMsg::Name {
                    name: "handle".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(record.owner, Some("bob".to_string()));
    }

    #[test]
    fn renews_and_releases_by_owner_only() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(10, "uatom")),
            ExecuteMsg::Register {
                name: "club".to_string(),
                value: "v1".to_string(),
            },
        )
        .unwrap();
        let unauthorized = execute(
            deps.as_mut(),
            env_at(120),
            mock_info("bob", &coins(10, "uatom")),
            ExecuteMsg::Renew {
                name: "club".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::NotNameOwner {}));

        execute(
            deps.as_mut(),
            env_at(150),
            mock_info("alice", &coins(10, "uatom")),
            ExecuteMsg::Renew {
                name: "club".to_string(),
            },
        )
        .unwrap();
        let record: NameResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(150),
                QueryMsg::Name {
                    name: "club".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(record.expires_at, Some(300));

        execute(
            deps.as_mut(),
            env_at(160),
            mock_info("alice", &[]),
            ExecuteMsg::Release {
                name: "club".to_string(),
            },
        )
        .unwrap();
        let released: NameResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(160),
                QueryMsg::Name {
                    name: "club".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(released.owner, None);
    }

    #[test]
    fn owner_withdraws_collected_fees() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(10, "uatom")),
            ExecuteMsg::Register {
                name: "alice".to_string(),
                value: "addr1".to_string(),
            },
        )
        .unwrap();
        let response = execute(
            deps.as_mut(),
            env_at(110),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(6),
            },
        )
        .unwrap();
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(6, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), env_at(110), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(4));
    }
}
