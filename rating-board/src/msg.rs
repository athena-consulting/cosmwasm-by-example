use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub max_rating: Option<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateItem {
        name: String,
        description: Option<String>,
    },
    RateItem {
        item_id: u64,
        rating: u8,
    },
    UpdateConfig {
        max_rating: Option<u8>,
        new_owner: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ItemResponse)]
    Item { item_id: u64 },
    #[returns(RatingResponse)]
    Rating { item_id: u64, rater: String },
    #[returns(ItemsResponse)]
    Items {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub max_rating: u8,
    pub next_item_id: u64,
}

#[cw_serde]
pub struct ItemResponse {
    pub item_id: u64,
    pub creator: String,
    pub name: String,
    pub description: Option<String>,
    pub total_rating: u64,
    pub rating_count: u64,
    pub average_rating: Option<String>,
}

#[cw_serde]
pub struct RatingResponse {
    pub item_id: u64,
    pub rater: String,
    pub rating: u8,
}

#[cw_serde]
pub struct ItemsResponse {
    pub items: Vec<ItemResponse>,
}
