#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use sha2::{Digest, Sha256};

use crate::error::ContractError;
use crate::msg::{EscrowResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Escrow, ESCROW};

const CONTRACT_NAME: &str = "crates.io:hashlock-escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if info.funds.len() != 1 {
        return Err(ContractError::SingleCoinRequired {});
    }

    let recipient = deps.api.addr_validate(&msg.recipient)?;
    let escrow = Escrow {
        sender: info.sender.clone(),
        recipient: recipient.clone(),
        hash: msg.hash,
        deadline: msg.deadline,
        amount: info.funds[0].clone(),
        claimed: false,
        refunded: false,
    };
    ESCROW.save(deps.storage, &escrow)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("deadline", msg.deadline.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim { preimage } => claim(deps, env, info, preimage),
        ExecuteMsg::Refund {} => refund(deps, env, info),
    }
}

pub fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    preimage: Binary,
) -> Result<Response, ContractError> {
    let mut escrow = ESCROW.load(deps.storage)?;

    if escrow.claimed || escrow.refunded {
        return Err(ContractError::EscrowClosed {});
    }
    if info.sender != escrow.recipient {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time >= escrow.deadline {
        return Err(ContractError::Expired {});
    }
    if preimage.as_slice().is_empty() {
        return Err(ContractError::EmptyPreimage {});
    }

    let hash = Sha256::digest(preimage.as_slice());
    if hash.as_slice() != escrow.hash.as_slice() {
        return Err(ContractError::InvalidPreimage {});
    }

    escrow.claimed = true;
    ESCROW.save(deps.storage, &escrow)?;

    Ok(
        send_response("claim", escrow.recipient.to_string(), escrow.amount)
            .add_attribute("preimage", preimage.to_base64()),
    )
}

pub fn refund(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut escrow = ESCROW.load(deps.storage)?;

    if escrow.claimed || escrow.refunded {
        return Err(ContractError::EscrowClosed {});
    }
    if info.sender != escrow.sender {
        return Err(ContractError::Unauthorized {});
    }
    if env.block.time < escrow.deadline {
        return Err(ContractError::NotExpired {});
    }

    escrow.refunded = true;
    ESCROW.save(deps.storage, &escrow)?;

    Ok(send_response(
        "refund",
        escrow.sender.to_string(),
        escrow.amount,
    ))
}

fn send_response(action: &str, to_address: String, amount: Coin) -> Response {
    Response::new()
        .add_message(BankMsg::Send {
            to_address: to_address.clone(),
            amount: vec![amount.clone()],
        })
        .add_attribute("action", action)
        .add_attribute("to", to_address)
        .add_attribute("amount", amount.amount.to_string())
        .add_attribute("denom", amount.denom)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Escrow {} => to_json_binary(&query_escrow(deps)?),
    }
}

pub fn query_escrow(deps: Deps) -> StdResult<EscrowResponse> {
    let escrow = ESCROW.load(deps.storage)?;
    Ok(EscrowResponse {
        sender: escrow.sender.to_string(),
        recipient: escrow.recipient.to_string(),
        hash: escrow.hash,
        deadline: escrow.deadline,
        amount: escrow.amount,
        claimed: escrow.claimed,
        refunded: escrow.refunded,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::testing::{MockApi, MockQuerier};
    use cosmwasm_std::{coin, coins, OwnedDeps, Timestamp};
    use cosmwasm_std::{Empty, MemoryStorage};

    fn hash(preimage: &[u8]) -> Binary {
        Binary::from(Sha256::digest(preimage).to_vec())
    }

    fn instantiate_default(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
        instantiate(
            deps,
            env,
            mock_info("sender", &coins(500, "uconst")),
            InstantiateMsg {
                recipient: "recipient".to_string(),
                hash: hash(b"open sesame"),
                deadline: Timestamp::from_seconds(2_000),
            },
        )
    }

    fn setup() -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier, Empty>, Env) {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1_000);
        instantiate_default(deps.as_mut(), env.clone()).unwrap();
        (deps, env)
    }

    #[test]
    fn instantiate_records_single_coin_escrow() {
        let (deps, _) = setup();

        let escrow = query_escrow(deps.as_ref()).unwrap();

        assert_eq!(escrow.sender, "sender");
        assert_eq!(escrow.recipient, "recipient");
        assert_eq!(escrow.amount, coin(500, "uconst"));
        assert!(!escrow.claimed);
        assert!(!escrow.refunded);
    }

    #[test]
    fn recipient_claims_before_deadline_with_valid_preimage() {
        let (mut deps, env) = setup();

        let res = claim(
            deps.as_mut(),
            env,
            mock_info("recipient", &[]),
            Binary::from(b"open sesame".as_slice()),
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.attributes[0].value, "claim");

        let escrow = ESCROW.load(&deps.storage).unwrap();
        assert!(escrow.claimed);
        assert!(!escrow.refunded);
    }

    #[test]
    fn invalid_preimage_is_rejected() {
        let (mut deps, env) = setup();

        let err = claim(
            deps.as_mut(),
            env,
            mock_info("recipient", &[]),
            Binary::from(b"wrong".as_slice()),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::InvalidPreimage {});
    }

    #[test]
    fn sender_refunds_after_deadline() {
        let (mut deps, mut env) = setup();
        env.block.time = Timestamp::from_seconds(2_001);

        let res = refund(deps.as_mut(), env, mock_info("sender", &[])).unwrap();

        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.attributes[0].value, "refund");

        let escrow = ESCROW.load(&deps.storage).unwrap();
        assert!(!escrow.claimed);
        assert!(escrow.refunded);
    }

    #[test]
    fn cannot_refund_before_deadline() {
        let (mut deps, env) = setup();

        let err = refund(deps.as_mut(), env, mock_info("sender", &[])).unwrap_err();

        assert_eq!(err, ContractError::NotExpired {});
    }
}
