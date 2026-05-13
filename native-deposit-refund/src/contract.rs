#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, DepositResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Deposit, CONFIG, DEPOSITS};

const CONTRACT_NAME: &str = "crates.io:native-deposit-refund";
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
    if msg.deposit_amount.is_zero() {
        return Err(ContractError::ZeroDeposit {});
    }

    let admin = match msg.admin {
        Some(admin) => deps.api.addr_validate(&admin)?,
        None => info.sender,
    };

    let config = Config {
        admin,
        denom: msg.denom,
        deposit_amount: msg.deposit_amount,
        refund_deadline: msg.refund_deadline,
    };

    CONFIG.save(deps.storage, &config)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("denom", config.denom)
        .add_attribute("deposit_amount", config.deposit_amount)
        .add_attribute("refund_deadline", config.refund_deadline.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit {} => execute_deposit(deps, env, info),
        ExecuteMsg::Refund {} => execute_refund(deps, env, info),
        ExecuteMsg::Forfeit {
            depositor,
            recipient,
        } => execute_forfeit(deps, env, info, depositor, recipient),
    }
}

fn execute_deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_exact_deposit(&info, &config)?;

    if DEPOSITS.has(deps.storage, &info.sender) {
        return Err(ContractError::AlreadyDeposited {});
    }

    let deposit = Deposit {
        amount: config.deposit_amount,
        deposited_at: env.block.time.seconds(),
    };
    DEPOSITS.save(deps.storage, &info.sender, &deposit)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("depositor", info.sender)
        .add_attribute("amount", deposit.amount)
        .add_attribute("denom", config.denom))
}

fn execute_refund(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if env.block.time.seconds() > config.refund_deadline {
        return Err(ContractError::RefundClosed {});
    }

    let deposit = DEPOSITS
        .may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::NoDeposit {})?;
    DEPOSITS.remove(deps.storage, &info.sender);

    let refund = Coin::new(deposit.amount.u128(), config.denom.clone());
    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![refund],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "refund")
        .add_attribute("depositor", info.sender)
        .add_attribute("amount", deposit.amount)
        .add_attribute("denom", config.denom))
}

fn execute_forfeit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    depositor: String,
    recipient: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time.seconds() <= config.refund_deadline {
        return Err(ContractError::DeadlineNotReached {});
    }

    let depositor_addr = deps.api.addr_validate(&depositor)?;
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    let deposit = DEPOSITS
        .may_load(deps.storage, &depositor_addr)?
        .ok_or(ContractError::NoDeposit {})?;
    DEPOSITS.remove(deps.storage, &depositor_addr);

    let forfeited = Coin::new(deposit.amount.u128(), config.denom.clone());
    let msg = BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: vec![forfeited],
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "forfeit")
        .add_attribute("depositor", depositor_addr)
        .add_attribute("recipient", recipient_addr)
        .add_attribute("amount", deposit.amount)
        .add_attribute("denom", config.denom))
}

fn assert_exact_deposit(info: &MessageInfo, config: &Config) -> Result<(), ContractError> {
    if info.funds.len() != 1 {
        return Err(ContractError::InvalidDeposit {});
    }

    let paid = &info.funds[0];
    if paid.denom != config.denom || paid.amount != config.deposit_amount {
        return Err(ContractError::InvalidDeposit {});
    }

    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<QueryResponse, StdError> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Deposit { address } => to_binary(&query_deposit(deps, address)?),
    }
}

fn query_config(deps: Deps) -> Result<ConfigResponse, StdError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        denom: config.denom,
        deposit_amount: config.deposit_amount,
        refund_deadline: config.refund_deadline,
    })
}

fn query_deposit(deps: Deps, address: String) -> Result<DepositResponse, StdError> {
    let address = deps.api.addr_validate(&address)?;
    let deposit = DEPOSITS.may_load(deps.storage, &address)?;

    Ok(match deposit {
        Some(deposit) => DepositResponse {
            address: address.to_string(),
            has_deposit: true,
            amount: deposit.amount,
            deposited_at: Some(deposit.deposited_at),
        },
        None => DepositResponse {
            address: address.to_string(),
            has_deposit: false,
            amount: Default::default(),
            deposited_at: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, BankMsg, CosmosMsg, Timestamp, Uint128};

    use super::*;

    fn instantiate_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            admin: Some("admin".to_string()),
            denom: "uatom".to_string(),
            deposit_amount: Uint128::new(100),
            refund_deadline: 1_000,
        };
        instantiate(deps, mock_env(), mock_info("creator", &[]), msg).unwrap();
    }

    #[test]
    fn deposit_requires_exact_native_payment() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(99, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDeposit {});

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[coin(100, "uatom"), coin(1, "uosmo")]),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidDeposit {});
    }

    #[test]
    fn deposit_saves_one_record_per_user() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();
        assert_eq!(res.attributes[0].value, "deposit");

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::AlreadyDeposited {});

        let deposit = query_deposit(deps.as_ref(), "alice".to_string()).unwrap();
        assert!(deposit.has_deposit);
        assert_eq!(deposit.amount, Uint128::new(100));
    }

    #[test]
    fn user_can_refund_before_deadline() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(999);
        let res = execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::Refund {},
        )
        .unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "alice".to_string(),
                amount: coins(100, "uatom"),
            })
        );

        let deposit = query_deposit(deps.as_ref(), "alice".to_string()).unwrap();
        assert!(!deposit.has_deposit);
    }

    #[test]
    fn refund_closes_after_deadline() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_001);
        let err = execute(
            deps.as_mut(),
            env,
            mock_info("alice", &[]),
            ExecuteMsg::Refund {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::RefundClosed {});
    }

    #[test]
    fn admin_can_forfeit_after_deadline() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_001);
        let res = execute(
            deps.as_mut(),
            env,
            mock_info("admin", &[]),
            ExecuteMsg::Forfeit {
                depositor: "alice".to_string(),
                recipient: "treasury".to_string(),
            },
        )
        .unwrap();

        assert_eq!(
            res.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: "treasury".to_string(),
                amount: coins(100, "uatom"),
            })
        );
    }

    #[test]
    fn forfeit_requires_admin_and_deadline() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::Deposit {},
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Forfeit {
                depositor: "alice".to_string(),
                recipient: "treasury".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        let mut before_deadline = mock_env();
        before_deadline.block.time = Timestamp::from_seconds(999);
        let err = execute(
            deps.as_mut(),
            before_deadline,
            mock_info("admin", &[]),
            ExecuteMsg::Forfeit {
                depositor: "alice".to_string(),
                recipient: "treasury".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::DeadlineNotReached {});
    }
}
