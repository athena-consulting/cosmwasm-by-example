#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, EscrowResponse, ExecuteMsg, InstantiateMsg, NextEscrowIdResponse, QueryMsg,
};
use crate::state::{Config, Escrow, EscrowStatus, CONFIG, ESCROWS, NEXT_ID};

const CONTRACT_NAME: &str = "crates.io:native-milestone-escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.denom.trim().is_empty() {
        return Err(ContractError::EmptyDenom {});
    }

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender.clone(),
    };
    let default_arbiter = match msg.default_arbiter {
        Some(arbiter) => Some(deps.api.addr_validate(&arbiter)?),
        None => None,
    };

    let config = Config {
        owner,
        denom: msg.denom,
        default_arbiter,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    NEXT_ID.save(deps.storage, &1)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", config.owner)
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
        ExecuteMsg::CreateEscrow {
            id,
            seller,
            arbiter,
            amount,
            description,
            timeout,
        } => execute_create_escrow(
            deps,
            env,
            info,
            CreateEscrowArgs {
                id,
                seller,
                arbiter,
                amount,
                description,
                timeout,
            },
        ),
        ExecuteMsg::Release { id } => execute_release(deps, env, info, id),
        ExecuteMsg::Refund { id } => execute_refund(deps, env, info, id),
        ExecuteMsg::UpdateConfig {
            owner,
            default_arbiter,
        } => execute_update_config(deps, info, owner, default_arbiter),
    }
}

struct CreateEscrowArgs {
    id: Option<u64>,
    seller: String,
    arbiter: Option<String>,
    amount: Uint128,
    description: String,
    timeout: Option<u64>,
}

fn execute_create_escrow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    args: CreateEscrowArgs,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if args.amount.is_zero() {
        return Err(ContractError::InvalidAmount {});
    }
    if let Some(timeout) = args.timeout {
        if timeout <= env.block.time.seconds() {
            return Err(ContractError::InvalidTimeout {});
        }
    }
    assert_exact_payment(&info, &config.denom, args.amount)?;

    let id = match args.id {
        Some(id) => id,
        None => NEXT_ID.load(deps.storage)?,
    };
    if ESCROWS.has(deps.storage, id) {
        return Err(ContractError::DuplicateEscrow {});
    }

    let seller = deps.api.addr_validate(&args.seller)?;
    let arbiter = match args.arbiter {
        Some(arbiter) => deps.api.addr_validate(&arbiter)?,
        None => config
            .default_arbiter
            .ok_or(ContractError::MissingArbiter {})?,
    };

    ESCROWS.save(
        deps.storage,
        id,
        &Escrow {
            id,
            buyer: info.sender.clone(),
            seller: seller.clone(),
            arbiter: arbiter.clone(),
            amount: args.amount,
            description: args.description,
            timeout: args.timeout,
            created_at: env.block.time.seconds(),
            settled_at: None,
            status: EscrowStatus::Active,
        },
    )?;

    NEXT_ID.update(deps.storage, |next_id| -> StdResult<_> {
        Ok(next_id.max(id.saturating_add(1)))
    })?;

    Ok(Response::new()
        .add_attribute("action", "create_escrow")
        .add_attribute("id", id.to_string())
        .add_attribute("buyer", info.sender)
        .add_attribute("seller", seller)
        .add_attribute("arbiter", arbiter)
        .add_attribute("amount", args.amount)
        .add_attribute("denom", config.denom))
}

fn execute_release(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut escrow = ESCROWS.load(deps.storage, id)?;
    assert_active(&escrow)?;

    if info.sender != escrow.buyer && info.sender != escrow.arbiter {
        return Err(ContractError::Unauthorized {});
    }

    escrow.status = EscrowStatus::Released;
    escrow.settled_at = Some(env.block.time.seconds());
    ESCROWS.save(deps.storage, id, &escrow)?;

    let send = CosmosMsg::Bank(BankMsg::Send {
        to_address: escrow.seller.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount: escrow.amount,
        }],
    });

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "release")
        .add_attribute("id", id.to_string())
        .add_attribute("to", escrow.seller)
        .add_attribute("amount", escrow.amount)
        .add_attribute("denom", config.denom))
}

fn execute_refund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut escrow = ESCROWS.load(deps.storage, id)?;
    assert_active(&escrow)?;

    let timed_out = escrow
        .timeout
        .map(|timeout| env.block.time.seconds() >= timeout)
        .unwrap_or(false);
    let can_refund = info.sender == escrow.arbiter
        || info.sender == escrow.seller
        || (info.sender == escrow.buyer && timed_out);
    if !can_refund {
        return Err(ContractError::Unauthorized {});
    }

    escrow.status = EscrowStatus::Refunded;
    escrow.settled_at = Some(env.block.time.seconds());
    ESCROWS.save(deps.storage, id, &escrow)?;

    let send = CosmosMsg::Bank(BankMsg::Send {
        to_address: escrow.buyer.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount: escrow.amount,
        }],
    });

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "refund")
        .add_attribute("id", id.to_string())
        .add_attribute("to", escrow.buyer)
        .add_attribute("amount", escrow.amount)
        .add_attribute("denom", config.denom))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    default_arbiter: Option<String>,
) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut config| -> Result<_, ContractError> {
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = deps.api.addr_validate(&owner)?;
        }
        if let Some(default_arbiter) = default_arbiter {
            config.default_arbiter = Some(deps.api.addr_validate(&default_arbiter)?);
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Escrow { id } => to_json_binary(&query_escrow(deps, id)?),
        QueryMsg::NextEscrowId {} => to_json_binary(&NextEscrowIdResponse {
            next_id: NEXT_ID.load(deps.storage)?,
        }),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        denom: config.denom,
        default_arbiter: config.default_arbiter,
    })
}

fn query_escrow(deps: Deps, id: u64) -> StdResult<EscrowResponse> {
    let escrow = ESCROWS.load(deps.storage, id)?;
    Ok(EscrowResponse {
        id: escrow.id,
        buyer: escrow.buyer,
        seller: escrow.seller,
        arbiter: escrow.arbiter,
        amount: escrow.amount,
        description: escrow.description,
        timeout: escrow.timeout,
        created_at: escrow.created_at,
        settled_at: escrow.settled_at,
        status: escrow.status,
    })
}

fn assert_active(escrow: &Escrow) -> Result<(), ContractError> {
    if escrow.status != EscrowStatus::Active {
        return Err(ContractError::AlreadySettled {});
    }
    Ok(())
}

fn assert_exact_payment(
    info: &MessageInfo,
    denom: &str,
    expected: Uint128,
) -> Result<(), ContractError> {
    if info.funds.len() != 1 || info.funds[0].denom != denom || info.funds[0].amount != expected {
        let actual = info
            .funds
            .iter()
            .find(|coin| coin.denom == denom)
            .map(|coin| coin.amount)
            .unwrap_or_default();
        return Err(ContractError::InvalidPayment {
            expected,
            actual,
            denom: denom.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_json, Addr, BankMsg, CosmosMsg, Timestamp};

    use super::*;

    fn setup() -> (
        cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        Env,
    ) {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(100);

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            InstantiateMsg {
                denom: "uatom".to_string(),
                owner: None,
                default_arbiter: Some("arbiter".to_string()),
            },
        )
        .unwrap();

        (deps, env)
    }

    fn create_escrow(deps: DepsMut, env: Env, timeout: Option<u64>) -> Response {
        execute(
            deps,
            env,
            mock_info("buyer", &coins(50, "uatom")),
            ExecuteMsg::CreateEscrow {
                id: None,
                seller: "seller".to_string(),
                arbiter: None,
                amount: Uint128::new(50),
                description: "website milestone".to_string(),
                timeout,
            },
        )
        .unwrap()
    }

    #[test]
    fn buyer_creates_funded_escrow() {
        let (mut deps, env) = setup();

        let res = create_escrow(deps.as_mut(), env.clone(), Some(200));
        assert_eq!(res.attributes[0].value, "create_escrow");

        let escrow: EscrowResponse =
            from_json(query(deps.as_ref(), env.clone(), QueryMsg::Escrow { id: 1 }).unwrap())
                .unwrap();
        assert_eq!(escrow.buyer, Addr::unchecked("buyer"));
        assert_eq!(escrow.seller, Addr::unchecked("seller"));
        assert_eq!(escrow.arbiter, Addr::unchecked("arbiter"));
        assert_eq!(escrow.amount, Uint128::new(50));
        assert_eq!(escrow.timeout, Some(200));
        assert_eq!(escrow.status, EscrowStatus::Active);

        let next: NextEscrowIdResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::NextEscrowId {}).unwrap()).unwrap();
        assert_eq!(next.next_id, 2);
    }

    #[test]
    fn create_requires_exact_payment_and_future_timeout() {
        let (mut deps, env) = setup();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("buyer", &coins(49, "uatom")),
            ExecuteMsg::CreateEscrow {
                id: None,
                seller: "seller".to_string(),
                arbiter: None,
                amount: Uint128::new(50),
                description: "underpaid".to_string(),
                timeout: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidPayment {
                expected: Uint128::new(50),
                actual: Uint128::new(49),
                denom: "uatom".to_string(),
            }
        );

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("buyer", &coins(50, "uatom")),
            ExecuteMsg::CreateEscrow {
                id: None,
                seller: "seller".to_string(),
                arbiter: None,
                amount: Uint128::new(50),
                description: "expired".to_string(),
                timeout: Some(100),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidTimeout {});
    }

    #[test]
    fn release_sends_funds_to_seller_and_marks_settled() {
        let (mut deps, env) = setup();
        create_escrow(deps.as_mut(), env.clone(), None);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("buyer", &[]),
            ExecuteMsg::Release { id: 1 },
        )
        .unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "seller".to_string(),
                amount: vec![coin(50, "uatom")],
            })
        );

        let escrow: EscrowResponse =
            from_json(query(deps.as_ref(), env.clone(), QueryMsg::Escrow { id: 1 }).unwrap())
                .unwrap();
        assert_eq!(escrow.status, EscrowStatus::Released);
        assert_eq!(escrow.settled_at, Some(100));

        let err = execute(
            deps.as_mut(),
            env,
            mock_info("arbiter", &[]),
            ExecuteMsg::Refund { id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AlreadySettled {});
    }

    #[test]
    fn refund_permissions_include_timeout_for_buyer() {
        let (mut deps, mut env) = setup();
        create_escrow(deps.as_mut(), env.clone(), Some(150));

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("buyer", &[]),
            ExecuteMsg::Refund { id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        env.block.time = Timestamp::from_seconds(150);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("buyer", &[]),
            ExecuteMsg::Refund { id: 1 },
        )
        .unwrap();
        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "buyer".to_string(),
                amount: vec![coin(50, "uatom")],
            })
        );

        let escrow: EscrowResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Escrow { id: 1 }).unwrap()).unwrap();
        assert_eq!(escrow.status, EscrowStatus::Refunded);
    }

    #[test]
    fn arbiter_and_seller_can_refund_before_timeout() {
        let (mut deps, env) = setup();
        create_escrow(deps.as_mut(), env.clone(), Some(200));

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("seller", &[]),
            ExecuteMsg::Refund { id: 1 },
        )
        .unwrap();

        create_escrow(deps.as_mut(), env.clone(), Some(200));
        let res = execute(
            deps.as_mut(),
            env,
            mock_info("arbiter", &[]),
            ExecuteMsg::Refund { id: 2 },
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "refund");
    }

    #[test]
    fn owner_updates_default_arbiter() {
        let (mut deps, env) = setup();

        let err = execute(
            deps.as_mut(),
            env.clone(),
            mock_info("buyer", &[]),
            ExecuteMsg::UpdateConfig {
                owner: None,
                default_arbiter: Some("new_arbiter".to_string()),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                owner: Some("new_owner".to_string()),
                default_arbiter: Some("new_arbiter".to_string()),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.owner, Addr::unchecked("new_owner"));
        assert_eq!(config.default_arbiter, Some(Addr::unchecked("new_arbiter")));
    }
}
