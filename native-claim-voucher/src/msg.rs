use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateVoucher {
        id: String,
        recipient: String,
        expires_at_height: u64,
    },
    Claim {
        id: String,
    },
    RefundExpired {
        id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    #[returns(VoucherResponse)]
    Voucher { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoucherResponse {
    pub id: String,
    pub creator: String,
    pub recipient: String,
    pub funds: Vec<Coin>,
    pub expires_at_height: u64,
    pub status: String,
}
