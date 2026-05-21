use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetProfile {
        display_name: String,
        bio: Option<String>,
        website: Option<String>,
    },
    ClearProfile {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetProfileResponse)]
    GetProfile { address: String },
    #[returns(ListProfilesResponse)]
    ListProfiles {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ProfileResponse {
    pub owner: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub website: Option<String>,
}

#[cw_serde]
pub struct GetProfileResponse {
    pub profile: Option<ProfileResponse>,
}

#[cw_serde]
pub struct ListProfilesResponse {
    pub profiles: Vec<ProfileResponse>,
}
