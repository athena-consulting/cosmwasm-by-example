# Vault Contract

This contract represents a vault that allows users to deposit and withdraw tokens. It uses the cw20 token standard and maintains a mapping of user balances.

## Overview

The `Vault` contract is deployed with a specific cw20 token address, which is passed as a parameter to the constructor. It provides the following functionalities:

- `deposit`: Allows users to deposit tokens into the vault. The number of shares to be minted for the user is calculated based on the deposited amount and the existing token balance of the vault.


- `withdraw`: Allows users to withdraw tokens from the vault. The number of tokens to be withdrawn is calculated based on the number of shares the user wants to burn and the current token balance of the vault.


## Contract Functions

### Deposit

```rust
pub fn execute_deposit(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let token_info=TOKEN_INFO.load(deps.storage)?;
        let mut total_supply=TOTAL_SUPPLY.load(deps.storage)?;
        let mut shares=Uint128::zero();
        let mut balance=BALANCE_OF.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        let balance_of=get_token_balance_of(&deps, info.sender.clone(), token_info.token_address.clone())?;
        if total_supply.is_zero() {
            shares+=amount;
        }
        else {
            shares+=amount.checked_mul(total_supply).map_err(StdError::overflow)?.checked_div(balance_of).map_err(StdError::divide_by_zero)?
        }
        
        // Giving allowance to this contract
        give_allowance(env.clone(), info.clone(), amount, token_info.token_address.clone())?;
        total_supply+=shares;
        TOTAL_SUPPLY.save(deps.storage, &total_supply)?;
        balance+=shares;
        BALANCE_OF.save(deps.storage, info.sender.clone(), &balance)?;

        let transfer_from_msg=cw20::Cw20ExecuteMsg::TransferFrom { owner: info.sender.to_string(), recipient: env.contract.address.to_string(), amount };
        let msg=CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute { contract_addr: token_info.token_address.to_string(), msg: to_binary(&transfer_from_msg)?, funds: info.funds });

        Ok(Response::new().add_attribute("action", "deposit").add_message(msg))
        
    }
```

This function allows users to deposit tokens into the vault.

Parameters:
- `amount`: The amount of tokens to deposit.

### Withdraw

```rust
 pub fn execute_withdraw(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        shares: Uint128,
    ) -> Result<Response, ContractError> {
        let token_info=TOKEN_INFO.load(deps.storage)?;
        let mut total_supply=TOTAL_SUPPLY.load(deps.storage)?;
        let mut balance=BALANCE_OF.load(deps.storage, info.sender.clone()).unwrap_or(Uint128::zero());
        let balance_of=get_token_balance_of(&deps, info.sender.clone(), token_info.token_address.clone())?;
        let amount=shares.checked_mul(balance_of).map_err(StdError::overflow)?.checked_div(total_supply).map_err(StdError::divide_by_zero)?;
        total_supply-=shares;
        TOTAL_SUPPLY.save(deps.storage, &total_supply)?;
        balance-=shares;
        BALANCE_OF.save(deps.storage, info.sender.clone(), &balance)?;

        let transfer_msg=cw20::Cw20ExecuteMsg::Transfer { recipient: info.sender.to_string(), amount};
        let msg=CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute { contract_addr: token_info.token_address.to_string(), msg: to_binary(&transfer_msg)?, funds: info.funds });

        Ok(Response::new().add_attribute("action", "withdraw").add_message(msg))
    }
    
```

This function allows users to withdraw tokens from the vault.

Parameters:
- `shares`: The number of shares to burn for token withdrawal.

---