use cosmwasm_std::{to_binary, Binary, Coin, CosmosMsg, StdError, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub denom: String,
    pub goal: Uint128,
    pub start: Option<Timestamp>,
    pub deadline: Timestamp,
    pub name: String,
    pub description: String,
    pub execute_msg: Option<CosmosMsg>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // fund the project with a given amount of tokens
    // receives coins from `WasmExecuteMsg.funds`
    Fund {},
    // execute the project if the goal is reached
    Execute {},
    // refund the project if the goal is not reached
    Refund {},
    // claim the project's funds if the goal is reached
    Claim {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    // * `get_shares`: returns a user's shares in the project.
    GetShares {
        user: String,
    },
    // returns a list of all funders and their shares.
    GetFunders {
        limit: Uint128,
        start_after: Option<String>,
    },
    // returns total fund held by contract.
    GetTotalFunds {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)] // returns config
pub struct GetConfigResponse {
    pub goal: Coin,
    pub deadline: Timestamp,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)] // returns a user's shares in the project.
pub struct GetSharesResponse {
    pub address: String,
    pub shares: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)] // returns a list of all funders and their shares.
pub struct GetFundersResponse {
    pub funders: Vec<(String, Uint128)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)] // Get Total Funds Response
pub struct GetTotalFundsResponse {
    pub total_funds: Coin,
}

#[derive(Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryResponseWrapper {
    GetConfigResponse(GetConfigResponse),
    GetSharesResponse(GetSharesResponse),
    GetFundersResponse(GetFundersResponse),
    GetTotalFundsResponse(GetTotalFundsResponse),
}

impl QueryResponseWrapper {
    pub fn to_binary(&self) -> Result<Binary, StdError> {
        match self {
            QueryResponseWrapper::GetConfigResponse(x) => to_binary(x),
            QueryResponseWrapper::GetSharesResponse(x) => to_binary(x),
            QueryResponseWrapper::GetFundersResponse(x) => to_binary(x),
            QueryResponseWrapper::GetTotalFundsResponse(x) => to_binary(x),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
