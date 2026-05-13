#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, GiftResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, Gift, GiftStatus, CONFIG, GIFTS, NEXT_GIFT_ID};

const CONTRACT_NAME: &str = "crates.io:native-gift-card";
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
    NEXT_GIFT_ID.save(deps.storage, &1)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("denom", denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateGift { recipient } => execute_create_gift(deps, info, recipient),
        ExecuteMsg::Claim { gift_id } => execute_claim(deps, info, gift_id),
        ExecuteMsg::Cancel { gift_id } => execute_cancel(deps, info, gift_id),
    }
}

fn execute_create_gift(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let coin = require_single_coin(&info.funds, &config.denom)?;
    let recipient = deps.api.addr_validate(&recipient)?;
    let gift_id = NEXT_GIFT_ID.load(deps.storage)?;

    GIFTS.save(
        deps.storage,
        gift_id,
        &Gift {
            creator: info.sender.clone(),
            recipient: recipient.clone(),
            amount: coin.amount,
            status: GiftStatus::Open,
        },
    )?;
    NEXT_GIFT_ID.save(deps.storage, &(gift_id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_gift")
        .add_attribute("gift_id", gift_id.to_string())
        .add_attribute("creator", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", coin.amount))
}

fn execute_claim(
    deps: DepsMut,
    info: MessageInfo,
    gift_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut gift = GIFTS.load(deps.storage, gift_id)?;
    if info.sender != gift.recipient {
        return Err(ContractError::NotRecipient {});
    }
    assert_open(&gift)?;

    gift.status = GiftStatus::Claimed;
    GIFTS.save(deps.storage, gift_id, &gift)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: gift.recipient.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: gift.amount,
            }],
        })
        .add_attribute("action", "claim")
        .add_attribute("gift_id", gift_id.to_string())
        .add_attribute("recipient", gift.recipient)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", gift.amount))
}

fn execute_cancel(
    deps: DepsMut,
    info: MessageInfo,
    gift_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut gift = GIFTS.load(deps.storage, gift_id)?;
    if info.sender != gift.creator {
        return Err(ContractError::NotCreator {});
    }
    assert_open(&gift)?;

    gift.status = GiftStatus::Canceled;
    GIFTS.save(deps.storage, gift_id, &gift)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: gift.creator.to_string(),
            amount: vec![Coin {
                denom: config.denom.clone(),
                amount: gift.amount,
            }],
        })
        .add_attribute("action", "cancel")
        .add_attribute("gift_id", gift_id.to_string())
        .add_attribute("creator", gift.creator)
        .add_attribute("denom", config.denom)
        .add_attribute("amount", gift.amount))
}

fn require_single_coin<'a>(
    funds: &'a [Coin],
    expected_denom: &str,
) -> Result<&'a Coin, ContractError> {
    match funds {
        [] => Err(ContractError::MissingGiftFunds {}),
        [coin] => {
            if coin.denom != expected_denom {
                return Err(ContractError::WrongDenom {
                    expected: expected_denom.to_string(),
                });
            }
            if coin.amount.is_zero() {
                return Err(ContractError::InvalidGiftAmount {});
            }
            Ok(coin)
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}

fn assert_open(gift: &Gift) -> Result<(), ContractError> {
    if matches!(gift.status, GiftStatus::Open) {
        Ok(())
    } else {
        Err(ContractError::GiftSettled {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Gift { gift_id } => to_binary(&query_gift(deps, gift_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let next_gift_id = NEXT_GIFT_ID.load(deps.storage)?;
    Ok(ConfigResponse {
        denom: config.denom,
        next_gift_id,
    })
}

fn query_gift(deps: Deps, gift_id: u64) -> StdResult<GiftResponse> {
    let config = CONFIG.load(deps.storage)?;
    let gift = GIFTS.load(deps.storage, gift_id)?;
    Ok(GiftResponse {
        gift_id,
        creator: gift.creator.to_string(),
        recipient: gift.recipient.to_string(),
        denom: config.denom,
        amount: gift.amount,
        status: match gift.status {
            GiftStatus::Open => "open",
            GiftStatus::Claimed => "claimed",
            GiftStatus::Canceled => "canceled",
        }
        .to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, CosmosMsg, Uint128};

    fn instantiate_contract(deps: DepsMut) {
        instantiate(
            deps,
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                denom: "uatom".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn instantiate_rejects_empty_denom_and_sets_next_id() {
        let mut deps = mock_dependencies();

        let err = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &[]),
            InstantiateMsg {
                denom: "".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EmptyDenom {}));

        instantiate_contract(deps.as_mut());

        let response: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(response.denom, "uatom");
        assert_eq!(response.next_gift_id, 1);
    }

    #[test]
    fn create_gift_requires_single_positive_configured_coin() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CreateGift {
                recipient: "bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MissingGiftFunds {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[coin(10, "uosmo")]),
            ExecuteMsg::CreateGift {
                recipient: "bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::WrongDenom { .. }));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[coin(10, "uatom"), coin(1, "uosmo")]),
            ExecuteMsg::CreateGift {
                recipient: "bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MultipleDenoms {}));

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[coin(0, "uatom")]),
            ExecuteMsg::CreateGift {
                recipient: "bob".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidGiftAmount {}));
    }

    #[test]
    fn create_gift_stores_record_and_advances_id() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[coin(42, "uatom")]),
            ExecuteMsg::CreateGift {
                recipient: "bob".to_string(),
            },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes[1].value, "1");

        let gift: GiftResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Gift { gift_id: 1 }).unwrap())
                .unwrap();
        assert_eq!(gift.creator, "alice");
        assert_eq!(gift.recipient, "bob");
        assert_eq!(gift.amount, Uint128::new(42));
        assert_eq!(gift.status, "open");

        let config: ConfigResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.next_gift_id, 2);
    }

    #[test]
    fn only_recipient_can_claim_once() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[coin(42, "uatom")]),
            ExecuteMsg::CreateGift {
                recipient: "bob".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Claim { gift_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotRecipient {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Claim { gift_id: 1 },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "bob");
                assert_eq!(amount, &coins(42, "uatom"));
            }
            _ => panic!("expected bank send"),
        }

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Claim { gift_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::GiftSettled {}));
    }

    #[test]
    fn only_creator_can_cancel_unclaimed_gift() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[coin(55, "uatom")]),
            ExecuteMsg::CreateGift {
                recipient: "bob".to_string(),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::Cancel { gift_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotCreator {}));

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::Cancel { gift_id: 1 },
        )
        .unwrap();
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "alice");
                assert_eq!(amount, &coins(55, "uatom"));
            }
            _ => panic!("expected bank send"),
        }

        let gift: GiftResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Gift { gift_id: 1 }).unwrap())
                .unwrap();
        assert_eq!(gift.status, "canceled");
    }
}
