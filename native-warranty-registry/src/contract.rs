#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ClaimResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, ProductResponse, QueryMsg,
};
use crate::state::{ClaimStatus, Config, Product, WarrantyClaim, CLAIMS, CONFIG, PRODUCTS};

const CONTRACT_NAME: &str = "crates.io:native-warranty-registry";
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
        return Err(ContractError::InvalidRegistrationFee {});
    }
    if msg.coverage_seconds == 0 {
        return Err(ContractError::InvalidCoveragePeriod {});
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
            coverage_seconds: msg.coverage_seconds,
            next_claim_id: 1,
            balance: Uint128::zero(),
        },
    )?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("denom", denom)
        .add_attribute("registration_fee", msg.registration_fee)
        .add_attribute("coverage_seconds", msg.coverage_seconds.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RegisterProduct { serial, metadata } => {
            execute_register_product(deps, env, info, serial, metadata)
        }
        ExecuteMsg::TransferProduct { serial, new_owner } => {
            execute_transfer_product(deps, env, info, serial, new_owner)
        }
        ExecuteMsg::FileClaim { serial, note } => execute_file_claim(deps, env, info, serial, note),
        ExecuteMsg::ResolveClaim { claim_id, approved } => {
            execute_resolve_claim(deps, info, claim_id, approved)
        }
        ExecuteMsg::Withdraw { recipient, amount } => {
            execute_withdraw(deps, info, recipient, amount)
        }
    }
}

fn execute_register_product(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    serial: String,
    metadata: String,
) -> Result<Response, ContractError> {
    let serial = normalize_serial(&serial)?;
    let metadata = validate_text(metadata, ContractError::EmptyMetadata {})?;
    let mut config = CONFIG.load(deps.storage)?;
    require_payment(&info.funds, &config.denom, config.registration_fee)?;

    if let Some(product) = PRODUCTS.may_load(deps.storage, &serial)? {
        if !is_expired(&product, &env) {
            return Err(ContractError::ProductCovered {});
        }
    }

    let expires_at = env
        .block
        .time
        .seconds()
        .checked_add(config.coverage_seconds)
        .ok_or_else(|| StdError::generic_err("coverage expiry overflow"))?;
    PRODUCTS.save(
        deps.storage,
        &serial,
        &Product {
            serial: serial.clone(),
            owner: info.sender.clone(),
            metadata,
            expires_at,
        },
    )?;
    config.balance = config
        .balance
        .checked_add(config.registration_fee)
        .map_err(StdError::overflow)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "register_product")
        .add_attribute("serial", serial)
        .add_attribute("owner", info.sender)
        .add_attribute("expires_at", expires_at.to_string())
        .add_attribute("balance", config.balance))
}

fn execute_transfer_product(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    serial: String,
    new_owner: String,
) -> Result<Response, ContractError> {
    let serial = normalize_serial(&serial)?;
    let mut product = PRODUCTS.load(deps.storage, &serial)?;
    if product.owner != info.sender {
        return Err(ContractError::NotProductOwner {});
    }
    if is_expired(&product, &env) {
        return Err(ContractError::CoverageExpired {});
    }
    product.owner = deps.api.addr_validate(&new_owner)?;
    PRODUCTS.save(deps.storage, &serial, &product)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_product")
        .add_attribute("serial", serial)
        .add_attribute("new_owner", product.owner))
}

fn execute_file_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    serial: String,
    note: String,
) -> Result<Response, ContractError> {
    let serial = normalize_serial(&serial)?;
    let note = validate_text(note, ContractError::EmptyClaimNote {})?;
    let product = PRODUCTS.load(deps.storage, &serial)?;
    if product.owner != info.sender {
        return Err(ContractError::NotProductOwner {});
    }
    if is_expired(&product, &env) {
        return Err(ContractError::CoverageExpired {});
    }

    let mut config = CONFIG.load(deps.storage)?;
    let claim_id = config.next_claim_id;
    CLAIMS.save(
        deps.storage,
        claim_id,
        &WarrantyClaim {
            id: claim_id,
            serial: serial.clone(),
            claimant: info.sender.clone(),
            note,
            status: ClaimStatus::Open,
        },
    )?;
    config.next_claim_id = config
        .next_claim_id
        .checked_add(1)
        .ok_or_else(|| StdError::generic_err("claim id overflow"))?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "file_claim")
        .add_attribute("claim_id", claim_id.to_string())
        .add_attribute("serial", serial)
        .add_attribute("claimant", info.sender))
}

fn execute_resolve_claim(
    deps: DepsMut,
    info: MessageInfo,
    claim_id: u64,
    approved: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let mut claim = CLAIMS.load(deps.storage, claim_id)?;
    if claim.status != ClaimStatus::Open {
        return Err(ContractError::ClaimAlreadyResolved {});
    }
    claim.status = if approved {
        ClaimStatus::Approved
    } else {
        ClaimStatus::Rejected
    };
    CLAIMS.save(deps.storage, claim_id, &claim)?;

    Ok(Response::new()
        .add_attribute("action", "resolve_claim")
        .add_attribute("claim_id", claim_id.to_string())
        .add_attribute("status", status_label(&claim.status)))
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

fn normalize_serial(serial: &str) -> Result<String, ContractError> {
    let serial = serial.trim().to_ascii_uppercase();
    if serial.is_empty() {
        return Err(ContractError::EmptySerial {});
    }
    Ok(serial)
}

fn validate_text(value: String, error: ContractError) -> Result<String, ContractError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(error);
    }
    Ok(value)
}

fn is_expired(product: &Product, env: &Env) -> bool {
    env.block.time.seconds() >= product.expires_at
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
        QueryMsg::Product { serial } => to_binary(&query_product(deps, env, serial)?),
        QueryMsg::Claim { claim_id } => to_binary(&query_claim(deps, claim_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        denom: config.denom,
        registration_fee: config.registration_fee,
        coverage_seconds: config.coverage_seconds,
        next_claim_id: config.next_claim_id,
        balance: config.balance,
    })
}

fn query_product(deps: Deps, env: Env, serial: String) -> StdResult<ProductResponse> {
    let serial = normalize_serial(&serial).map_err(|err| StdError::generic_err(err.to_string()))?;
    let product = PRODUCTS.may_load(deps.storage, &serial)?;
    Ok(match product {
        Some(product) => {
            let expired = is_expired(&product, &env);
            ProductResponse {
                serial,
                owner: Some(product.owner.to_string()),
                metadata: Some(product.metadata),
                expires_at: Some(product.expires_at),
                is_expired: expired,
            }
        }
        None => ProductResponse {
            serial,
            owner: None,
            metadata: None,
            expires_at: None,
            is_expired: false,
        },
    })
}

fn query_claim(deps: Deps, claim_id: u64) -> StdResult<ClaimResponse> {
    let claim = CLAIMS.load(deps.storage, claim_id)?;
    Ok(ClaimResponse {
        id: claim.id,
        serial: claim.serial,
        claimant: claim.claimant.to_string(),
        note: claim.note,
        status: status_label(&claim.status).to_string(),
    })
}

fn status_label(status: &ClaimStatus) -> &'static str {
    match status {
        ClaimStatus::Open => "open",
        ClaimStatus::Approved => "approved",
        ClaimStatus::Rejected => "rejected",
    }
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
                registration_fee: Uint128::new(15),
                coverage_seconds: 100,
            },
        )
        .unwrap();
    }

    fn register_product(deps: DepsMut) {
        execute(
            deps,
            env_at(100),
            mock_info("alice", &coins(15, "uatom")),
            ExecuteMsg::RegisterProduct {
                serial: "abc-123".to_string(),
                metadata: "laptop".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn registers_and_transfers_product() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        register_product(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(120),
            mock_info("alice", &[]),
            ExecuteMsg::TransferProduct {
                serial: "ABC-123".to_string(),
                new_owner: "bob".to_string(),
            },
        )
        .unwrap();
        let product: ProductResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(120),
                QueryMsg::Product {
                    serial: "abc-123".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(product.owner, Some("bob".to_string()));
        assert_eq!(product.expires_at, Some(200));
    }

    #[test]
    fn blocks_duplicate_registration_until_expired() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        register_product(deps.as_mut());

        let covered = execute(
            deps.as_mut(),
            env_at(150),
            mock_info("bob", &coins(15, "uatom")),
            ExecuteMsg::RegisterProduct {
                serial: "abc-123".to_string(),
                metadata: "tablet".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(covered, ContractError::ProductCovered {}));

        execute(
            deps.as_mut(),
            env_at(200),
            mock_info("bob", &coins(15, "uatom")),
            ExecuteMsg::RegisterProduct {
                serial: "abc-123".to_string(),
                metadata: "tablet".to_string(),
            },
        )
        .unwrap();
        let product: ProductResponse = from_binary(
            &query(
                deps.as_ref(),
                env_at(200),
                QueryMsg::Product {
                    serial: "abc-123".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(product.owner, Some("bob".to_string()));
    }

    #[test]
    fn owner_resolves_open_claims() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        register_product(deps.as_mut());

        execute(
            deps.as_mut(),
            env_at(120),
            mock_info("alice", &[]),
            ExecuteMsg::FileClaim {
                serial: "abc-123".to_string(),
                note: "screen failure".to_string(),
            },
        )
        .unwrap();
        let unauthorized = execute(
            deps.as_mut(),
            env_at(121),
            mock_info("intruder", &[]),
            ExecuteMsg::ResolveClaim {
                claim_id: 1,
                approved: true,
            },
        )
        .unwrap_err();
        assert!(matches!(unauthorized, ContractError::Unauthorized {}));

        execute(
            deps.as_mut(),
            env_at(122),
            mock_info("owner", &[]),
            ExecuteMsg::ResolveClaim {
                claim_id: 1,
                approved: true,
            },
        )
        .unwrap();
        let claim: ClaimResponse = from_binary(
            &query(deps.as_ref(), env_at(122), QueryMsg::Claim { claim_id: 1 }).unwrap(),
        )
        .unwrap();
        assert_eq!(claim.status, "approved");
    }

    #[test]
    fn owner_withdraws_collected_fees() {
        let mut deps = mock_dependencies();
        instantiate_contract(deps.as_mut());
        register_product(deps.as_mut());

        let response = execute(
            deps.as_mut(),
            env_at(130),
            mock_info("owner", &[]),
            ExecuteMsg::Withdraw {
                recipient: "treasury".to_string(),
                amount: Uint128::new(10),
            },
        )
        .unwrap();
        match &response.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, "treasury");
                assert_eq!(amount, &coins(10, "uatom"));
            }
            _ => panic!("unexpected message"),
        }
    }
}
