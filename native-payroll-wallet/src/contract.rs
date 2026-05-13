#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, EmployeeResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Employee, CONFIG, EMPLOYEES};

const CONTRACT_NAME: &str = "crates.io:native-payroll-wallet";
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
    if msg.period_seconds == 0 {
        return Err(ContractError::InvalidPeriod {});
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
            period_seconds: msg.period_seconds,
            balance: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("period_seconds", msg.period_seconds.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Fund {} => execute_fund(deps, info),
        ExecuteMsg::SetEmployee {
            employee,
            salary,
            last_claim,
        } => execute_set_employee(deps, info, employee, salary, last_claim),
        ExecuteMsg::RemoveEmployee { employee } => execute_remove_employee(deps, info, employee),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
    }
}

fn execute_fund(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    config.balance = config
        .balance
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "fund")
        .add_attribute("from", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("balance", config.balance))
}

fn execute_set_employee(
    deps: DepsMut,
    info: MessageInfo,
    employee: String,
    salary: Uint128,
    last_claim: Option<u64>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if salary.is_zero() {
        return Err(ContractError::InvalidSalary {});
    }
    let employee = deps.api.addr_validate(&employee)?;
    let last_claim = last_claim.unwrap_or_default();

    EMPLOYEES.save(deps.storage, &employee, &Employee { salary, last_claim })?;

    Ok(Response::new()
        .add_attribute("action", "set_employee")
        .add_attribute("employee", employee)
        .add_attribute("salary", salary)
        .add_attribute("last_claim", last_claim.to_string()))
}

fn execute_remove_employee(
    deps: DepsMut,
    info: MessageInfo,
    employee: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let employee = deps.api.addr_validate(&employee)?;
    EMPLOYEES.remove(deps.storage, &employee);

    Ok(Response::new()
        .add_attribute("action", "remove_employee")
        .add_attribute("employee", employee))
}

fn execute_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let mut employee = EMPLOYEES
        .may_load(deps.storage, &info.sender)?
        .ok_or(ContractError::EmployeeNotFound {})?;
    let next_claim = employee.last_claim + config.period_seconds;
    if env.block.time.seconds() < next_claim {
        return Err(ContractError::TooEarly {});
    }
    if config.balance < employee.salary {
        return Err(ContractError::InsufficientBalance {
            available: config.balance,
            required: employee.salary,
        });
    }

    config.balance = config
        .balance
        .checked_sub(employee.salary)
        .map_err(StdError::overflow)?;
    employee.last_claim = env.block.time.seconds();
    CONFIG.save(deps.storage, &config)?;
    EMPLOYEES.save(deps.storage, &info.sender, &employee)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: employee.salary,
            }],
        })
        .add_attribute("action", "claim")
        .add_attribute("employee", info.sender)
        .add_attribute("denom", config.denom)
        .add_attribute("salary", employee.salary)
        .add_attribute("balance", config.balance)
        .add_attribute("last_claim", employee.last_claim.to_string()))
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
        QueryMsg::Employee { employee } => to_binary(&query_employee(deps, employee)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        period_seconds: config.period_seconds,
        balance: config.balance,
    })
}

fn query_employee(deps: Deps, employee: String) -> StdResult<EmployeeResponse> {
    let config = CONFIG.load(deps.storage)?;
    let employee_addr = deps.api.addr_validate(&employee)?;
    let employee_state = EMPLOYEES.may_load(deps.storage, &employee_addr)?;
    let (salary, last_claim, configured) = match employee_state {
        Some(employee) => (employee.salary, employee.last_claim, true),
        None => (Uint128::zero(), 0, false),
    };

    Ok(EmployeeResponse {
        employee,
        configured,
        salary,
        last_claim,
        next_claim: last_claim + config.period_seconds,
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
                period_seconds: 100,
            },
        )
        .unwrap();
    }

    fn fund_and_set_employee(deps: DepsMut) {
        execute(
            deps,
            env_at(100),
            mock_info("funder", &coins(500, "uatom")),
            ExecuteMsg::Fund {},
        )
        .unwrap();
    }

    fn set_employee(deps: DepsMut) {
        execute(
            deps,
            env_at(100),
            mock_info("owner", &[]),
            ExecuteMsg::SetEmployee {
                employee: "alice".to_string(),
                salary: Uint128::new(125),
                last_claim: Some(100),
            },
        )
        .unwrap();
    }

    #[test]
    fn employee_claims_salary_after_period() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        fund_and_set_employee(deps.as_mut());
        set_employee(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            env_at(250),
            mock_info("alice", &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(125, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), env_at(250), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(375));
    }

    #[test]
    fn rejects_too_early_claim() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        fund_and_set_employee(deps.as_mut());
        set_employee(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            env_at(150),
            mock_info("alice", &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::TooEarly {}));
    }

    #[test]
    fn owner_controls_employee_records() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            env_at(100),
            mock_info("intruder", &[]),
            ExecuteMsg::SetEmployee {
                employee: "alice".to_string(),
                salary: Uint128::new(125),
                last_claim: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));

        set_employee(deps.as_mut());
        let employee: EmployeeResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(100),
                QueryMsg::Employee {
                    employee: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(employee.configured);
        assert_eq!(employee.salary, Uint128::new(125));
        assert_eq!(employee.next_claim, 200);
    }

    #[test]
    fn claim_requires_funded_payroll_balance() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        set_employee(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            env_at(250),
            mock_info("alice", &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InsufficientBalance { .. }));
    }
}
