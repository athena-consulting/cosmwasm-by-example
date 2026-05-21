use cosmwasm_schema::QueryResponses;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub name: String,
    pub description: Option<String>,
    pub issuers: Vec<String>,
    pub max_badges_per_holder: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateBadge {
        badge_id: String,
        title: String,
        description: String,
    },
    ArchiveBadge {
        badge_id: String,
    },
    AddIssuer {
        issuer: String,
    },
    RemoveIssuer {
        issuer: String,
    },
    AwardBadge {
        badge_id: String,
        recipient: String,
        note: Option<String>,
    },
    RevokeBadge {
        badge_id: String,
        holder: String,
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
    #[returns(IsIssuerResponse)]
    IsIssuer { address: String },
    #[returns(BadgeResponse)]
    Badge { badge_id: String },
    #[returns(BadgesResponse)]
    Badges {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(HolderBadgesResponse)]
    HolderBadges {
        holder: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(HasBadgeResponse)]
    HasBadge { holder: String, badge_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub max_badges_per_holder: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsIssuerResponse {
    pub address: String,
    pub is_issuer: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BadgeResponse {
    pub badge: BadgeInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BadgesResponse {
    pub badges: Vec<BadgeInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HolderBadgesResponse {
    pub holder: String,
    pub awards: Vec<AwardInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HasBadgeResponse {
    pub has_badge: bool,
    pub award: Option<AwardInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BadgeInfo {
    pub badge_id: String,
    pub title: String,
    pub description: String,
    pub creator: String,
    pub archived: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AwardInfo {
    pub badge_id: String,
    pub holder: String,
    pub awarded_by: String,
    pub awarded_at_height: u64,
    pub note: Option<String>,
}
