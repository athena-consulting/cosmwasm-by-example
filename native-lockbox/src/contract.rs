#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, LockResponse, NextIdResponse, QueryMsg,
};
use crate::state::{Config, Lock, CONFIG, LOCKS, NEXT_ID};

const CONTRACT_NAME: &str = "crates.io:native-lockbox";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let denom = msg.denom.trim().to_string();
    if denom.is_empty() {
        return Err(ContractError::EmptyDenom {});
    }

    CONFIG.save(
        deps.storage,
        &Config {
            denom: denom.clone(),
        },
    )?;
    NEXT_ID.save(deps.storage, &1)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("denom", denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Lock { unlock_time } => execute_lock(deps, env, info, unlock_time),
        ExecuteMsg::Extend { id, unlock_time } => execute_extend(deps, env, info, id, unlock_time),
        ExecuteMsg::Claim { id } => execute_claim(deps, env, info, id),
    }
}

fn execute_lock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    unlock_time: u64,
) -> Result<Response, ContractError> {
    if unlock_time <= env.block.time.seconds() {
        return Err(ContractError::UnlockNotFuture {});
    }
    let config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    let id = NEXT_ID.load(deps.storage)?;

    LOCKS.save(
        deps.storage,
        id,
        &Lock {
            owner: info.sender.clone(),
            amount: coin.amount,
            unlock_time,
            claimed: false,
        },
    )?;
    NEXT_ID.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "lock")
        .add_attribute("id", id.to_string())
        .add_attribute("owner", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("unlock_time", unlock_time.to_string()))
}

fn execute_extend(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    unlock_time: u64,
) -> Result<Response, ContractError> {
    if unlock_time <= env.block.time.seconds() {
        return Err(ContractError::UnlockNotFuture {});
    }
    let mut lock = LOCKS
        .may_load(deps.storage, id)?
        .ok_or(ContractError::LockNotFound {})?;
    ensure_owner_and_unclaimed(&info, &lock)?;
    if unlock_time <= lock.unlock_time {
        return Err(ContractError::UnlockNotFuture {});
    }

    lock.unlock_time = unlock_time;
    LOCKS.save(deps.storage, id, &lock)?;

    Ok(Response::new()
        .add_attribute("action", "extend")
        .add_attribute("id", id.to_string())
        .add_attribute("unlock_time", unlock_time.to_string()))
}

fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut lock = LOCKS
        .may_load(deps.storage, id)?
        .ok_or(ContractError::LockNotFound {})?;
    ensure_owner_and_unclaimed(&info, &lock)?;
    if env.block.time.seconds() < lock.unlock_time {
        return Err(ContractError::TooEarly {});
    }

    lock.claimed = true;
    LOCKS.save(deps.storage, id, &lock)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: lock.owner.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: lock.amount,
            }],
        })
        .add_attribute("action", "claim")
        .add_attribute("id", id.to_string())
        .add_attribute("owner", lock.owner)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", lock.amount))
}

fn ensure_owner_and_unclaimed(info: &MessageInfo, lock: &Lock) -> Result<(), ContractError> {
    if info.sender != lock.owner {
        return Err(ContractError::Unauthorized {});
    }
    if lock.claimed {
        return Err(ContractError::AlreadyClaimed {});
    }
    Ok(())
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
        QueryMsg::Lock { id } => to_binary(&query_lock(deps, id)?),
        QueryMsg::NextId {} => to_binary(&query_next_id(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        denom: config.denom,
    })
}

fn query_lock(deps: Deps, id: u64) -> StdResult<LockResponse> {
    let lock = LOCKS.load(deps.storage, id)?;
    Ok(LockResponse {
        id,
        owner: lock.owner.to_string(),
        amount: lock.amount,
        unlock_time: lock.unlock_time,
        claimed: lock.claimed,
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
                denom: "uatom".to_string(),
            },
        )
        .unwrap();
    }

    fn create_lock(deps: DepsMut) {
        execute(
            deps,
            env_at(100),
            mock_info("alice", &coins(88, "uatom")),
            ExecuteMsg::Lock { unlock_time: 200 },
        )
        .unwrap();
    }

    #[test]
    fn lock_records_deposit_and_next_id() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_lock(deps.as_mut());

        let lock: LockResponse =
            from_binary(&query(deps.as_ref(), env_at(100), QueryMsg::Lock { id: 1 }).unwrap())
                .unwrap();
        assert_eq!(lock.owner, "alice");
        assert_eq!(lock.amount.u128(), 88);
        assert_eq!(lock.unlock_time, 200);
        assert!(!lock.claimed);

        let next: NextIdResponse =
            from_binary(&query(deps.as_ref(), env_at(100), QueryMsg::NextId {}).unwrap()).unwrap();
        assert_eq!(next.next_id, 2);
    }

    #[test]
    fn owner_claims_after_unlock_time() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_lock(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            env_at(250),
            mock_info("alice", &[]),
            ExecuteMsg::Claim { id: 1 },
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(88, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }

    #[test]
    fn rejects_early_claim_and_non_owner() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_lock(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            env_at(150),
            mock_info("alice", &[]),
            ExecuteMsg::Claim { id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::TooEarly {}));

        let err = execute(
            deps.as_mut(),
            env_at(250),
            mock_info("bob", &[]),
            ExecuteMsg::Claim { id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn owner_can_extend_unclaimed_lock() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        create_lock(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(150),
            mock_info("alice", &[]),
            ExecuteMsg::Extend {
                id: 1,
                unlock_time: 300,
            },
        )
        .unwrap();

        let lock: LockResponse =
            from_binary(&query(deps.as_ref(), env_at(150), QueryMsg::Lock { id: 1 }).unwrap())
                .unwrap();
        assert_eq!(lock.unlock_time, 300);
    }
}
