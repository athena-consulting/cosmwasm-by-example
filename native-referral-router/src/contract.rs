#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "crates.io:native-referral-router";
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
    validate_referral_bps(msg.referral_bps)?;

    let owner = match msg.owner {
        Some(owner) => deps.api.addr_validate(&owner)?,
        None => info.sender,
    };
    let merchant = deps.api.addr_validate(&msg.merchant)?;

    CONFIG.save(
        deps.storage,
        &Config {
            owner: owner.clone(),
            merchant: merchant.clone(),
            denom: denom.clone(),
            referral_bps: msg.referral_bps,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("merchant", merchant)
        .add_attribute("denom", denom)
        .add_attribute("referral_bps", msg.referral_bps.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: cosmwasm_std::Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Pay { referrer } => execute_pay(deps, info, referrer),
        ExecuteMsg::UpdateConfig {
            merchant,
            referral_bps,
        } => execute_update_config(deps, info, merchant, referral_bps),
    }
}

fn execute_pay(
    deps: DepsMut,
    info: MessageInfo,
    referrer: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    let mut response = Response::new()
        .add_attribute("action", "pay")
        .add_attribute("payer", info.sender.clone())
        .add_attribute("merchant", config.merchant.clone())
        .add_attribute("denom", config.denom.clone())
        .add_attribute("amount", coin.amount);

    if let Some(referrer) = referrer {
        let referrer = deps.api.addr_validate(&referrer)?;
        if referrer == info.sender {
            return Err(ContractError::SelfReferral {});
        }
        let referral_bonus = coin
            .amount
            .multiply_ratio(config.referral_bps as u128, FULL_BPS as u128);
        if referral_bonus.is_zero() {
            return Err(ContractError::ReferralTooSmall {});
        }
        let merchant_amount = coin
            .amount
            .checked_sub(referral_bonus)
            .map_err(StdError::overflow)?;

        response = response
            .add_message(BankMsg::Send {
                to_address: referrer.to_string(),
                amount: vec![Coin {
                    denom: config.denom.clone(),
                    amount: referral_bonus,
                }],
            })
            .add_attribute("referrer", referrer)
            .add_attribute("referral_bonus", referral_bonus)
            .add_attribute("merchant_amount", merchant_amount)
            .add_message(BankMsg::Send {
                to_address: config.merchant.to_string(),
                amount: vec![Coin {
                    denom: config.denom,
                    amount: merchant_amount,
                }],
            });
    } else {
        response = response
            .add_attribute("merchant_amount", coin.amount)
            .add_message(BankMsg::Send {
                to_address: config.merchant.to_string(),
                amount: vec![Coin {
                    denom: config.denom,
                    amount: coin.amount,
                }],
            });
    }

    Ok(response)
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    merchant: Option<String>,
    referral_bps: Option<u16>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    if let Some(merchant) = merchant {
        config.merchant = deps.api.addr_validate(&merchant)?;
    }
    if let Some(referral_bps) = referral_bps {
        validate_referral_bps(referral_bps)?;
        config.referral_bps = referral_bps;
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("merchant", config.merchant)
        .add_attribute("referral_bps", config.referral_bps.to_string()))
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

fn validate_referral_bps(referral_bps: u16) -> Result<(), ContractError> {
    if referral_bps == 0 || referral_bps > FULL_BPS {
        return Err(ContractError::InvalidReferralBps {});
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
        merchant: config.merchant.to_string(),
        denom: config.denom,
        referral_bps: config.referral_bps,
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
                merchant: "merchant".to_string(),
                denom: "uatom".to_string(),
                referral_bps: 1000,
            },
        )
        .unwrap();
    }

    #[test]
    fn pay_without_referrer_sends_all_to_merchant() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &coins(100, "uatom")),
            ExecuteMsg::Pay { referrer: None },
        )
        .unwrap();

        assert_eq!(response.messages.len(), 1);
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "merchant");
                assert_eq!(amount, &coins(100, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }

    #[test]
    fn pay_with_referrer_splits_payment() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &coins(100, "uatom")),
            ExecuteMsg::Pay {
                referrer: Some("referrer".to_string()),
            },
        )
        .unwrap();

        assert_eq!(response.messages.len(), 2);
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "referrer");
                assert_eq!(amount, &coins(10, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
        match &response.messages[1].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "merchant");
                assert_eq!(amount, &coins(90, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }

    #[test]
    fn owner_updates_config() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner", &[]),
            ExecuteMsg::UpdateConfig {
                merchant: Some("newmerchant".to_string()),
                referral_bps: Some(2500),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.merchant, "newmerchant");
        assert_eq!(config.referral_bps, 2500);
    }

    #[test]
    fn rejects_bad_referrer_and_unauthorized_update() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("buyer", &coins(100, "uatom")),
            ExecuteMsg::Pay {
                referrer: Some("buyer".to_string()),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::SelfReferral {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("intruder", &[]),
            ExecuteMsg::UpdateConfig {
                merchant: None,
                referral_bps: Some(2500),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }
}
