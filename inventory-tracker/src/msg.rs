use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Uint128;
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
        sku: String,
        name: String,
        quantity: Uint128,
        note: Option<String>,
    },
    UpdateItem {
        item_id: u64,
        sku: String,
        name: String,
        note: Option<String>,
    },
    SetQuantity {
        item_id: u64,
        quantity: Uint128,
    },
    IncreaseQuantity {
        item_id: u64,
        amount: Uint128,
    },
    DecreaseQuantity {
        item_id: u64,
        amount: Uint128,
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
    pub item: InventoryItemResponse,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserItemsResponse {
    pub owner: String,
    pub items: Vec<InventoryItemResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ItemCountResponse {
    pub owner: String,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InventoryItemResponse {
    pub item_id: u64,
    pub owner: String,
    pub sku: String,
    pub name: String,
    pub quantity: Uint128,
    pub note: Option<String>,
    pub created_at_height: u64,
    pub updated_at_height: u64,
}
