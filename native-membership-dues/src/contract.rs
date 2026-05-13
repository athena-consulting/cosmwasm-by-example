#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MemberResponse, QueryMsg};
use crate::state::{Config, CONFIG, MEMBERS};

const CONTRACT_NAME: &str = "crates.io:native-membership-dues";
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
    validate_terms(msg.fee, msg.period_seconds)?;

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            denom: denom.clone(),
            fee: msg.fee,
            period_seconds: msg.period_seconds,
            balance: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("fee", msg.fee)
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
        ExecuteMsg::PayDues { member } => execute_pay_dues(deps, env, info, member),
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
        ExecuteMsg::UpdateTerms {
            fee,
            period_seconds,
        } => execute_update_terms(deps, info, fee, period_seconds),
    }
}

fn execute_pay_dues(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    member: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let coin = require_dues_payment(&info.funds, &config.denom, config.fee)?;
    let member = match member {
        Some(member) => deps.api.addr_validate(&member)?,
        None => info.sender.clone(),
    };

    let current_expiry = MEMBERS.may_load(deps.storage, &member)?.unwrap_or_default();
    let start = current_expiry.max(env.block.time.seconds());
    let expires_at = start + config.period_seconds;

    config.balance = config
        .balance
        .checked_add(coin.amount)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;
    MEMBERS.save(deps.storage, &member, &expires_at)?;

    Ok(Response::new()
        .add_attribute("action", "pay_dues")
        .add_attribute("payer", info.sender)
        .add_attribute("member", member)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount)
        .add_attribute("expires_at", expires_at.to_string())
        .add_attribute("balance", config.balance))
}

fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if amount.is_zero() {
        return Err(ContractError::InvalidWithdrawAmount {});
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

fn execute_update_terms(
    deps: DepsMut,
    info: MessageInfo,
    fee: Uint128,
    period_seconds: u64,
) -> Result<Response, ContractError> {
    validate_terms(fee, period_seconds)?;
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    config.fee = fee;
    config.period_seconds = period_seconds;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_terms")
        .add_attribute("fee", fee)
        .add_attribute("period_seconds", period_seconds.to_string()))
}

fn validate_terms(fee: Uint128, period_seconds: u64) -> Result<(), ContractError> {
    if fee.is_zero() {
        return Err(ContractError::InvalidFee {});
    }
    if period_seconds == 0 {
        return Err(ContractError::InvalidPeriod {});
    }
    Ok(())
}

fn require_dues_payment<'a>(
    funds: &'a [Coin],
    expected_denom: &str,
    required_fee: Uint128,
) -> Result<&'a Coin, ContractError> {
    match funds {
        [] => Err(ContractError::MissingPayment {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount < required_fee {
                return Err(ContractError::Underpaid {
                    paid: coin.amount,
                    required: required_fee,
                });
            }
            Ok(coin)
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Member { member } => to_binary(&query_member(deps, env, member)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        fee: config.fee,
        period_seconds: config.period_seconds,
        balance: config.balance,
    })
}

fn query_member(deps: Deps, env: Env, member: String) -> StdResult<MemberResponse> {
    let member_addr = deps.api.addr_validate(&member)?;
    let expires_at = MEMBERS
        .may_load(deps.storage, &member_addr)?
        .unwrap_or_default();
    Ok(MemberResponse {
        member,
        expires_at,
        active: expires_at > env.block.time.seconds(),
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
                fee: Uint128::new(100),
                period_seconds: 1_000,
            },
        )
        .unwrap();
    }

    #[test]
    fn pay_dues_creates_and_extends_membership() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::PayDues { member: None },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env_at(200),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::PayDues { member: None },
        )
        .unwrap();

        let member: MemberResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(200),
                QueryMsg::Member {
                    member: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert!(member.active);
        assert_eq!(member.expires_at, 2_100);

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), env_at(200), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(200));
    }

    #[test]
    fn payer_can_buy_dues_for_another_member() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("sponsor", &coins(150, "uatom")),
            ExecuteMsg::PayDues {
                member: Some("alice".to_string()),
            },
        )
        .unwrap();

        let member: MemberResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(100),
                QueryMsg::Member {
                    member: "alice".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(member.active);
        assert_eq!(member.expires_at, 1_100);
    }

    #[test]
    fn rejects_underpayment_and_wrong_denom() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let underpaid = execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(99, "uatom")),
            ExecuteMsg::PayDues { member: None },
        )
        .unwrap_err();
        assert!(matches!(underpaid, ContractError::Underpaid { .. }));

        let wrong_denom = execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(100, "uosmo")),
            ExecuteMsg::PayDues { member: None },
        )
        .unwrap_err();
        assert!(matches!(wrong_denom, ContractError::WrongDenom { .. }));
    }

    #[test]
    fn owner_withdraws_collected_dues() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(100),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::PayDues { member: None },
        )
        .unwrap();

        let unauthorized = execute(
            deps.as_mut(),
            env_at(100),
            mock_info("intruder", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(50),
            },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::Unauthorized {}));

        let response = execute(
            deps.as_mut(),
            env_at(100),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(40),
            },
        )
        .unwrap();

        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(40, "uatom"));
            }
            _ => panic!("unexpected message"),
        }

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), env_at(100), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.balance, Uint128::new(60));
    }
}
