use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::Announcement;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Publish {
        id: String,
        title: String,
        body: String,
        expires_at: u64,
    },
    Archive {
        id: String,
    },
    TransferOwnership {
        new_owner: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AnnouncementResponse)]
    Announcement { id: String },
    #[returns(ListAnnouncementsResponse)]
    List { active_only: Option<bool> },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
}

#[cw_serde]
pub struct AnnouncementResponse {
    pub announcement: Announcement,
    pub active: bool,
}

#[cw_serde]
pub struct ListAnnouncementsResponse {
    pub announcements: Vec<AnnouncementResponse>,
}
