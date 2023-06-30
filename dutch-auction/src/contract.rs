#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::State;
use cosmwasm_std::{
    coin, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, Uint128,entry_point
};
use cw721::Cw721ExecuteMsg::TransferNft;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        seller: info.sender,
        starting_price: Uint128::from(msg.starting_price),
        start_at: _env.block.time,
        expires_at: _env.block.time.plus_days(7), // 7 days
        discount_rate: Uint128::from(msg.discount_rate),
        nft_address: msg.nft_address,
        nft_id: Uint128::from(msg.nft_id),
        denom: msg.denom,
    };

    state.save(deps.storage).unwrap();

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::BuyNft {} => {
            let state = State::load(deps.storage).unwrap();
            let mut msgs: Vec<CosmosMsg> = Vec::new();

            // Check if the auction has expired
            if env.block.time >= state.expires_at {
                return Err(ContractError::AuctionExpired {});
            }

            // Calculate the current price based on the discount rate and time elapsed
            let current_price = calculate_current_price(
                state.starting_price,
                state.discount_rate,
                state.start_at.seconds(),
                env.block.time.seconds(),
            );

            // Check if the sent funds are enough to purchase the NFT
            if info.funds.is_empty() || info.funds[0].denom != state.denom {
                return Err(ContractError::InvalidDenomination {});
            }
            let sent_funds = info.funds[0].amount;
            if sent_funds < current_price {
                return Err(ContractError::InsufficientFunds {
                    expected: current_price,
                    actual: sent_funds,
                });
            }

            // Transfer the NFT from the seller to the buyer
            let transfer_nft_msg = TransferNft {
                recipient: info.sender.to_string(),
                token_id: state.nft_id.to_string(),
            };
            msgs.push(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: state.nft_address.to_string(),
                msg: to_binary(&transfer_nft_msg)?,
                funds: info.funds,
            }));

            // Calculate the refund amount and transfer it to the buyer
            let refund = sent_funds - current_price;
            if refund > Uint128::zero() {
                msgs.push(CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: vec![coin(refund.into(), state.denom.clone())],
                }));
            }

            // Remove the contract state after successful execution
            state.remove(deps.storage)?;

            Ok(Response::new()
                .add_attribute("action", "buy")
                .add_messages(msgs))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetPrice {} => {
            let state = State::load(deps.storage).unwrap();
            let current_price = calculate_current_price(
                state.starting_price,
                state.discount_rate,
                state.start_at.seconds(),
                _env.block.time.seconds(),
            );
            Ok(to_binary(&current_price).unwrap())
        }
    }
}

// Calculate the current price based on the discount rate and time elapsed
fn calculate_current_price(
    starting_price: Uint128,
    discount_rate: Uint128,
    start_at: u64,
    current_time: u64,
) -> Uint128 {
    let elapsed_time: u128 = (current_time - start_at).into();
    let discount = (elapsed_time * discount_rate.u128()) / (7 * 24 * 60 * 60);
    if discount >= starting_price.u128() {
        Uint128::zero()
    } else {
        starting_price - Uint128::from(discount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info
    };
    use cosmwasm_std::{coin, Addr, CosmosMsg, WasmMsg};
    use cw721::Cw721ExecuteMsg;

    const NFT_ADDRESS: &str = "nft_contract_address";
    const NFT_ID: u128 = 12345;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            starting_price: 100,
            discount_rate: 10,
            nft_address: Addr::unchecked(NFT_ADDRESS),
            nft_id: NFT_ID,
            denom: "uusd".to_string(),
        };

        let env = mock_env();
        let info = mock_info("creator", &[]);

        let res = instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let state = State::load(deps.as_ref().storage).unwrap();
        assert_eq!(state.seller, "creator");
        assert_eq!(state.starting_price, Uint128::new(100));
        assert_eq!(state.discount_rate, Uint128::new(10));
        assert_eq!(state.nft_address, NFT_ADDRESS.to_string());
        assert_eq!(state.nft_id, Uint128::new(12345));
        assert_eq!(state.denom, "uusd".to_string());
    }

    #[test]
    fn test_execute_buy_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("buyer", &[coin(100, "uusd")]);

        let starting_price = 100;
        let discount_rate = 10;

        let state = State {
            seller: Addr::unchecked("creator"),
            starting_price: Uint128::new(starting_price),
            start_at: env.block.time,
            expires_at: env.block.time.plus_seconds(1000),
            discount_rate: Uint128::new(discount_rate),
            nft_address: Addr::unchecked(NFT_ADDRESS),
            nft_id: Uint128::from(NFT_ID),
            denom: "uusd".to_string(),
        };
        state.save(deps.as_mut().storage).unwrap();

        let msg = ExecuteMsg::BuyNft {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Verify the response
        assert_eq!(1, res.messages.len());
        assert_eq!(res.attributes.len(), 1);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "buy");

        // Verify the NFT transfer message
        let expected_transfer_msg = Cw721ExecuteMsg::TransferNft {
            recipient: info.sender.to_string(),
            token_id: NFT_ID.to_string(),
        };
        let expected_messages = vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: NFT_ADDRESS.to_string(),
                msg: to_binary(&expected_transfer_msg).unwrap(),
                funds: vec![coin(100, "uusd")],
            })
        ];
        // Assert the expected response and attributes
        let expected_res = Response::new()
            .add_attribute("action", "buy")
            .add_messages(expected_messages);
        assert_eq!(res, expected_res);
    }

    #[test]
    fn test_execute_buy_nft_auction_expired() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("buyer", &[coin(100, "uusd")]);

        let starting_price = 100;
        let discount_rate = 10;

        let state = State {
            seller: Addr::unchecked("creator"),
            starting_price: Uint128::new(starting_price),
            start_at: env.block.time,
            expires_at: env.block.time.plus_seconds(1000),
            discount_rate: Uint128::new(discount_rate),
            nft_address: Addr::unchecked(NFT_ADDRESS),
            nft_id: Uint128::from(NFT_ID),
            denom: "uusd".to_string(),
        };
        state.save(deps.as_mut().storage).unwrap();

        env.block.time=env.block.time.plus_seconds(1001);
        let msg = ExecuteMsg::BuyNft {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        // Verify the error
        let expected_err = ContractError::AuctionExpired {};
        assert_eq!(res.unwrap_err(), expected_err);
    }

    #[test]
    fn test_execute_buy_nft_invalid_denomination() {
        let mut deps = mock_dependencies();
        let  env = mock_env();
        let info = mock_info("buyer", &[coin(100, "uusdt")]);

        let starting_price = 100;
        let discount_rate = 10;

        let state = State {
            seller: Addr::unchecked("creator"),
            starting_price: Uint128::new(starting_price),
            start_at: env.block.time,
            expires_at: env.block.time.plus_seconds(1000),
            discount_rate: Uint128::new(discount_rate),
            nft_address: Addr::unchecked(NFT_ADDRESS),
            nft_id: Uint128::from(NFT_ID),
            denom: "uusd".to_string(),
        };
        state.save(deps.as_mut().storage).unwrap();


        let msg = ExecuteMsg::BuyNft {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        // Verify the error
        let expected_err = ContractError::InvalidDenomination {};
        assert_eq!(res.unwrap_err(), expected_err);
    }

    #[test]
    fn test_execute_buy_nft_insufficient_funds() {
        let mut deps = mock_dependencies();
        let  env = mock_env();
        let info = mock_info("buyer", &[coin(50, "uusd")]);

        let starting_price = 100;
        let discount_rate = 10;

        let state = State {
            seller: Addr::unchecked("creator"),
            starting_price: Uint128::new(starting_price),
            start_at: env.block.time,
            expires_at: env.block.time.plus_seconds(1000),
            discount_rate: Uint128::new(discount_rate),
            nft_address: Addr::unchecked(NFT_ADDRESS),
            nft_id: Uint128::from(NFT_ID),
            denom: "uusd".to_string(),
        };
        state.save(deps.as_mut().storage).unwrap();


        let msg = ExecuteMsg::BuyNft {};
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        // Verify the error
        let expected_err = ContractError::InsufficientFunds {
            expected:Uint128::new(100),
            actual: Uint128::new(50),
        };
        assert_eq!(res.unwrap_err(), expected_err);
    }

}
