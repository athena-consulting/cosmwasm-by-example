# CW721 Base
The CW721 Base is the base contract of the CW721 spec, 
which is the Cosmwasm specification for non-fungible tokens.
The design is based on the [ERC721 standard](https://eips.ethereum.org/EIPS/eip-721).
This contract is the base contract which is the minimum functionalities fr the contract 
required to be able to be considered an NFT.

## Token ID
These tokens are non-fungible since they can be differentiated by the 
token ID, which does not exist in CW20 (fungible tokens). Each seperate
token should have unique idenitifier and an owner that currently owns the fungible token. 

## Functionalities
Below are the minimum functionalities that a contract should have to be considered a CW721.

### Transfer
All CW721 contracts should have a transfer message that allows the transfer of tokens 
from one address to another (or one contract to another). In some cases, it also may have approval
to transfer it from an owner. 

```rust
pub fn _transfer_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: &str,
        token_id: &str,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, &token_id)?;
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token)?;
        // set owner and remove existing approvals
        token.owner = deps.api.addr_validate(recipient)?;
        token.approvals = vec![];
        self.tokens.save(deps.storage, &token_id, &token)?;
        Ok(token)
    }
```

The function always needs to check first that the caller is able to send the NFT from the
owner (whether they are the owner or not).
The function then removes all approvals on a token, so that they do not stay with a different owner, and 
then changes the owner of the token ID to the recipient.

### Send NFT
Sending an NFT is basically a transfer an NFT to a contract, where the contract needs to aknowledge 
the NFT when received and execute a certain functionality. Similarly to CW20 tokens, the recipient 
contract must implement a `Cw721Receive` interface in order to be able to aknowledge the receival of NFTs.

```rust
fn send_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response<C>, ContractError> {
        // Transfer token
        self._transfer_nft(deps, &env, &info, &contract, &token_id)?;

        let send = Cw721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        };

        // Send message
        Ok(Response::new()
            .add_message(send.into_cosmos_msg(contract.clone())?)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", contract)
            .add_attribute("token_id", token_id))
    }
```


### Approve
An owner of a token ID or NFT can allow others to send their NFTs on their behalf using an
`Approve` message. The owner can add more than one approval and therefore more than one spender. 
When the NFT is sent, all approvals added are cleared. 

Approvals can be used for NFT marketplaces for example to allow the marketplace to send the token
from your behalf to a buyer.

There is also a functionality of approving all tokens that are owned by the owner.

```rust
pub fn _update_approvals(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: &str,
        token_id: &str,
        // if add == false, remove. if add == true, remove then set with this expiration
        add: bool,
        expires: Option<Expiration>,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, &token_id)?;
        // ensure we have permissions
        self.check_can_approve(deps.as_ref(), env, info, &token)?;

        // update the approval list (remove any for the same spender before adding)
        let spender_addr = deps.api.addr_validate(spender)?;
        token.approvals = token
            .approvals
            .into_iter()
            .filter(|apr| apr.spender != spender_addr)
            .collect();

        // only difference between approve and revoke
        if add {
            // reject expired data as invalid
            let expires = expires.unwrap_or_default();
            if expires.is_expired(&env.block) {
                return Err(ContractError::Expired {});
            }
            let approval = Approval {
                spender: spender_addr,
                expires,
            };
            token.approvals.push(approval);
        }

        self.tokens.save(deps.storage, &token_id, &token)?;

        Ok(token)
    }
```

### Revoke
Similarly to approving smart contracts, the owner can revoke a certain
approval anytime if deemed unnecessary or a risk. 
i.e A potential exploit in an NFT marketplace.

There is also a functionality of revoking all tokens that are owned by the owner.


# Expanding Further
Implementing on top of this contract allows to add metadata that gives different variables and
potentially an image for the NFT. Check for example [CW721 with Metadata](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-metadata-onchain)