use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddMember { address: String, label: String },
    RemoveMember { address: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MemberResponse)]
    IsMember { address: String },
}

#[cw_serde]
pub struct MemberResponse {
    pub is_member: bool,
    pub label: Option<String>,
}
