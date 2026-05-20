#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VestingResponse};
use crate::state::{Vesting, VESTING};

const CONTRACT_NAME: &str = "crates.io:native-vesting-wallet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if info.funds.len() != 1 || info.funds[0].amount.is_zero() {
        return Err(ContractError::InvalidFunds {});
    }
    if msg.start_height > msg.cliff_height || msg.cliff_height >= msg.end_height {
        return Err(ContractError::InvalidSchedule {});
    }

    let beneficiary = deps.api.addr_validate(&msg.beneficiary)?;
    VESTING.save(
        deps.storage,
        &Vesting {
            creator: info.sender.clone(),
            beneficiary: beneficiary.clone(),
            total: info.funds[0].clone(),
            claimed: Uint128::zero(),
            start_height: msg.start_height,
            cliff_height: msg.cliff_height,
            end_height: msg.end_height,
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender)
        .add_attribute("beneficiary", beneficiary)
        .add_attribute("denom", info.funds[0].denom.clone())
        .add_attribute("total", info.funds[0].amount))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Vesting {} => to_json_binary(&query_vesting(deps, env)?),
    }
}

fn execute_claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut vesting = VESTING.load(deps.storage)?;
    if info.sender != vesting.beneficiary {
        return Err(ContractError::Unauthorized {});
    }

    let claimable = claimable_amount(&vesting, env.block.height);
    if claimable.is_zero() {
        return Err(ContractError::NothingToClaim {});
    }

    vesting.claimed += claimable;
    VESTING.save(deps.storage, &vesting)?;

    Ok(Response::new()
        .add_message(BankMsg::Send {
            to_address: vesting.beneficiary.to_string(),
            amount: vec![Coin {
                denom: vesting.total.denom,
                amount: claimable,
            }],
        })
        .add_attribute("action", "claim")
        .add_attribute("beneficiary", info.sender)
        .add_attribute("amount", claimable))
}

fn query_vesting(deps: Deps, env: Env) -> StdResult<VestingResponse> {
    let vesting = VESTING.load(deps.storage)?;
    let vested = vested_amount(&vesting, env.block.height);
    Ok(VestingResponse {
        creator: vesting.creator.to_string(),
        beneficiary: vesting.beneficiary.to_string(),
        denom: vesting.total.denom,
        total: vesting.total.amount,
        claimed: vesting.claimed,
        vested,
        claimable: vested.saturating_sub(vesting.claimed),
        start_height: vesting.start_height,
        cliff_height: vesting.cliff_height,
        end_height: vesting.end_height,
    })
}

fn claimable_amount(vesting: &Vesting, height: u64) -> Uint128 {
    vested_amount(vesting, height).saturating_sub(vesting.claimed)
}

fn vested_amount(vesting: &Vesting, height: u64) -> Uint128 {
    if height < vesting.cliff_height {
        return Uint128::zero();
    }
    if height >= vesting.end_height {
        return vesting.total.amount;
    }

    let elapsed = height.saturating_sub(vesting.start_height);
    let duration = vesting.end_height - vesting.start_height;
    vesting.total.amount.multiply_ratio(elapsed, duration)
}
