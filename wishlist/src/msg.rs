use cosmwasm_schema::QueryResponses;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub max_items_per_user: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddItem {
        title: String,
        description: Option<String>,
        url: Option<String>,
        priority: u8,
    },
    UpdateItem {
        item_id: u64,
        title: String,
        description: Option<String>,
        url: Option<String>,
        priority: u8,
    },
    SetPurchased {
        item_id: u64,
        purchased: bool,
    },
    DeleteItem {
        item_id: u64,
    },
    UpdateConfig {
        max_items_per_user: u32,
    },
    TransferOwnership {
        new_owner: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ItemResponse)]
    Item { owner: String, item_id: u64 },
    #[returns(UserItemsResponse)]
    UserItems {
        owner: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(ItemCountResponse)]
    ItemCount { owner: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub max_items_per_user: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ItemResponse {
    pub item: WishItemResponse,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserItemsResponse {
    pub owner: String,
    pub items: Vec<WishItemResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ItemCountResponse {
    pub owner: String,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WishItemResponse {
    pub item_id: u64,
    pub owner: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub priority: u8,
    pub purchased: bool,
    pub created_at_height: u64,
    pub updated_at_height: u64,
}
