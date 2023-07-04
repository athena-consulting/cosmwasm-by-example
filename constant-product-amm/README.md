# WasmSwap

This contract is an Constant product automatic market maker (AMM) for the cosmwasm smart contract engine.
This contract allows you to swap tokens. Liquidity providers can add liquidity to the market and receive a 0.03% fee on every transaction.
The code also includes various error handling and validation checks to ensure the correctness and security of the operations.

# Instantiation

The contract can be instantiated with the following messages

```
{
    "token1_denom": {"cw20": "<CONTRACT_ADDRESS>"},
    "token2_denom": {"cw20": "<CONTRACT_ADDRESS>"},
}
```

Token denom can be  `cw20` for cw20 tokens. `cw20` tokens have a contract address. `CW20_CODE_ID` is the code id for a basic cw20 binary.

# Messages

### Add Liquidity

Allows a user to add liquidity to the pool.

```rust
pub fn execute_add_liquidity(
    deps: DepsMut,
    info: &MessageInfo,
    env: Env,
    min_liquidity: Uint128,
    token1_amount: Uint128,
    token2_amount: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &env.block)?;

    let token1 = TOKEN1.load(deps.storage)?;
    let token2 = TOKEN2.load(deps.storage)?;

    let mut token_supply = TOTAL_STORED.load(deps.storage)?;
    let liquidity_amount = token1_amount+token2_amount;
    token_supply+=liquidity_amount;
    TOTAL_STORED.save(deps.storage, &token_supply)?;

    if liquidity_amount < min_liquidity {
        return Err(ContractError::MinLiquidityError {
            min_liquidity,
            liquidity_available: liquidity_amount,
        });
    }

    // Generate cw20 transfer messages if necessary
    let mut transfer_msgs: Vec<CosmosMsg> = vec![];
    if let Cw20(addr) = token1.denom {
        transfer_msgs.push(get_cw20_transfer_from_msg(
            &info.sender,
            &env.contract.address,
            &addr,
            token1_amount,
        )?)
    }
    if let Cw20(addr) = token2.denom.clone() {
        transfer_msgs.push(get_cw20_transfer_from_msg(
            &info.sender,
            &env.contract.address,
            &addr,
            token2_amount,
        )?)
    }


    TOKEN1.update(deps.storage, |mut token1| -> Result<_, ContractError> {
        token1.reserve += token1_amount;
        Ok(token1)
    })?;
    TOKEN2.update(deps.storage, |mut token2| -> Result<_, ContractError> {
        token2.reserve += token2_amount;
        Ok(token2)
    })?;

    Ok(Response::new()
        .add_messages(transfer_msgs)
        .add_attributes(vec![
            attr("token1_amount", token1_amount),
            attr("token2_amount", token2_amount),
            attr("liquidity_received", liquidity_amount),
        ]))
}
```
Users can add liquidity to the AMM by calling the execute_add_liquidity function. This function takes the desired amounts of two tokens (`token1_amount` and `token2_amount`) and mints a corresponding amount of liquidity tokens. The liquidity tokens represent the user's share in the AMM's liquidity pool. The function also transfers the input tokens from the user to the contract.

### Remove Liquidity

Allows a user to remove liquidity from the pool.

```rust
pub fn execute_remove_liquidity(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    amount: Uint128,
    min_token1: Uint128,
    min_token2: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &env.block)?;


    let total_token_supply = TOTAL_STORED.load(deps.storage)?;
    let token1 = TOKEN1.load(deps.storage)?;
    let token2 = TOKEN2.load(deps.storage)?;

    if amount > total_token_supply {
        return Err(ContractError::InsufficientLiquidityError {
            requested: amount,
            available: total_token_supply,
        });
    }

    let token1_amount = amount
        .checked_mul(token1.reserve)
        .map_err(StdError::overflow)?
        .checked_div(total_token_supply)
        .map_err(StdError::divide_by_zero)?;
    if token1_amount < min_token1 {
        return Err(ContractError::MinToken1Error {
            requested: min_token1,
            available: token1_amount,
        });
    }

    let token2_amount = amount
        .checked_mul(token2.reserve)
        .map_err(StdError::overflow)?
        .checked_div(total_token_supply)
        .map_err(StdError::divide_by_zero)?;
    if token2_amount < min_token2 {
        return Err(ContractError::MinToken2Error {
            requested: min_token2,
            available: token2_amount,
        });
    }

    TOKEN1.update(deps.storage, |mut token1| -> Result<_, ContractError> {
        token1.reserve = token1
            .reserve
            .checked_sub(token1_amount)
            .map_err(StdError::overflow)?;
        Ok(token1)
    })?;

    TOKEN2.update(deps.storage, |mut token2| -> Result<_, ContractError> {
        token2.reserve = token2
            .reserve
            .checked_sub(token2_amount)
            .map_err(StdError::overflow)?;
        Ok(token2)
    })?;

    let token1_transfer_msg = match token1.denom {
        Denom::Cw20(addr) => get_cw20_transfer_to_msg(&info.sender, &addr, token1_amount)?,
        Denom::Native(_denom) => {unimplemented!()},
    };
    let token2_transfer_msg = match token2.denom {
        Denom::Cw20(addr) => get_cw20_transfer_to_msg(&info.sender, &addr, token2_amount)?,
        Denom::Native(_denom) => {unimplemented!()},
    };

    Ok(Response::new()
        .add_messages(vec![
            token1_transfer_msg,
            token2_transfer_msg,
        ])
        .add_attributes(vec![
            attr("liquidity_burned", amount),
            attr("token1_returned", token1_amount),
            attr("token2_returned", token2_amount),
        ]))
}
```
Liquidity providers can remove their liquidity by calling the execute_remove_liquidity function. They specify the amount of liquidity tokens (amount) they want to burn, and the function calculates the proportionate amounts of the underlying tokens (token1_amount and token2_amount). The function transfers the corresponding tokens to the user and decreases the token reserves accordingly.


### Swap

Swap one asset for the other

```rust
pub fn execute_swap(
    deps: DepsMut,
    info: &MessageInfo,
    input_amount: Uint128,
    _env: Env,
    input_token_enum: TokenSelect,
    recipient: String,
    min_token: Uint128,
    expiration: Option<Expiration>,
) -> Result<Response, ContractError> {
    check_expiration(&expiration, &_env.block)?;

    let input_token_item = match input_token_enum {
        TokenSelect::Token1 => TOKEN1,
        TokenSelect::Token2 => TOKEN2,
    };
    let input_token = input_token_item.load(deps.storage)?;
    let output_token_item = match input_token_enum {
        TokenSelect::Token1 => TOKEN2,
        TokenSelect::Token2 => TOKEN1,
    };
    let output_token = output_token_item.load(deps.storage)?;


    let fees = FEES.load(deps.storage)?;
    let total_fee_percent = fees.lp_fee_percent + fees.protocol_fee_percent;
    let token_bought = get_input_price(
        input_amount,
        input_token.reserve,
        output_token.reserve,
        total_fee_percent,
    )?;

    if min_token > token_bought {
        return Err(ContractError::SwapMinError {
            min: min_token,
            available: token_bought,
        });
    }
    // Calculate fees
    let protocol_fee_amount = get_protocol_fee_amount(input_amount, fees.protocol_fee_percent)?;
    let input_amount_minus_protocol_fee = input_amount - protocol_fee_amount;

    let mut msgs = match input_token.denom.clone() {
        Denom::Cw20(addr) => vec![get_cw20_transfer_from_msg(
            &info.sender,
            &_env.contract.address,
            &addr,
            input_amount_minus_protocol_fee,
        )?],
        Denom::Native(_) => vec![],
    };

    // Send protocol fee to protocol fee recipient
    if !protocol_fee_amount.is_zero() {
        msgs.push(get_fee_transfer_msg(
            &info.sender,
            &fees.protocol_fee_recipient,
            &input_token.denom,
            protocol_fee_amount,
        )?)
    }

    let recipient = deps.api.addr_validate(&recipient)?;
    // Create transfer to message
    msgs.push(match output_token.denom {
        Denom::Cw20(addr) => get_cw20_transfer_to_msg(&recipient, &addr, token_bought)?,
        Denom::Native(_denom) => {unimplemented!()},
    });

    input_token_item.update(
        deps.storage,
        |mut input_token| -> Result<_, ContractError> {
            input_token.reserve = input_token
                .reserve
                .checked_add(input_amount_minus_protocol_fee)
                .map_err(StdError::overflow)?;
            Ok(input_token)
        },
    )?;

    output_token_item.update(
        deps.storage,
        |mut output_token| -> Result<_, ContractError> {
            output_token.reserve = output_token
                .reserve
                .checked_sub(token_bought)
                .map_err(StdError::overflow)?;
            Ok(output_token)
        },
    )?;

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        attr("native_sold", input_amount),
        attr("token_bought", token_bought),
    ]))
}
```
Users can swap tokens using the AMM by calling the `execute_swap` function. They specify the input token (`input_token`), the amount to swap (`input_amount`), and the minimum output amount (min_output). The function calculates the output amount based on the constant product formula and checks if it meets the minimum requirement. If the swap is valid, it transfers the input token from the user to the contract and transfers the output token back to the user.

### Configuration Update

To update the AMM configuration

```rust
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Option<String>,
    lp_fee_percent: Decimal,
    protocol_fee_percent: Decimal,
    protocol_fee_recipient: String,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if Some(info.sender) != owner {
        return Err(ContractError::Unauthorized {});
    }

    let new_owner_addr = new_owner
        .as_ref()
        .map(|h| deps.api.addr_validate(h))
        .transpose()?;
    OWNER.save(deps.storage, &new_owner_addr)?;

    let total_fee_percent = lp_fee_percent + protocol_fee_percent;
    let max_fee_percent = Decimal::from_str(MAX_FEE_PERCENT)?;
    if total_fee_percent > max_fee_percent {
        return Err(ContractError::FeesTooHigh {
            max_fee_percent,
            total_fee_percent,
        });
    }

    let protocol_fee_recipient = deps.api.addr_validate(&protocol_fee_recipient)?;
    let updated_fees = Fees {
        protocol_fee_recipient: protocol_fee_recipient.clone(),
        lp_fee_percent,
        protocol_fee_percent,
    };
    FEES.save(deps.storage, &updated_fees)?;

    let new_owner = new_owner.unwrap_or_default();
    Ok(Response::new().add_attributes(vec![
        attr("new_owner", new_owner),
        attr("lp_fee_percent", lp_fee_percent.to_string()),
        attr("protocol_fee_percent", protocol_fee_percent.to_string()),
        attr("protocol_fee_recipient", protocol_fee_recipient.to_string()),
    ]))
}
```
The AMM's configuration can be updated by the owner using the `execute_update_config` function. The owner can change the LP (liquidity provider) fee percentage, the protocol fee percentage, and the protocol fee recipient address.

### Deposit Freezing

To freeze the deposit to AMM

```rust
fn execute_freeze_deposits(
    deps: DepsMut,
    sender: Addr,
    freeze: bool,
) -> Result<Response, ContractError> {
    if let Some(owner) = OWNER.load(deps.storage)? {
        if sender != owner {
            return Err(ContractError::UnauthorizedPoolFreeze {});
        }
    } else {
        return Err(ContractError::UnauthorizedPoolFreeze {});
    }

    FROZEN.save(deps.storage, &freeze)?;
    Ok(Response::new().add_attribute("action", "freezing-contracts"))
}

fn check_expiration(
    expiration: &Option<Expiration>,
    block: &BlockInfo,
) -> Result<(), ContractError> {
    match expiration {
        Some(e) => {
            if e.is_expired(block) {
                return Err(ContractError::MsgExpirationError {});
            }
            Ok(())
        }
        None => Ok(()),
    }
}
```
The owner can freeze deposits to the AMM by calling the `execute_freeze_deposits` function. This prevents users from adding liquidity or swapping tokens. Only the owner can freeze or unfreeze deposits.

