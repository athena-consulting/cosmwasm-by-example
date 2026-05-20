use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddContact {
        name: String,
        address: String,
        note: Option<String>,
    },
    RemoveContact {
        name: String,
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
    #[returns(ContactResponse)]
    Contact { name: String },
    #[returns(ContactsResponse)]
    Contacts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
}

#[cw_serde]
pub struct ContactResponse {
    pub name: String,
    pub address: String,
    pub note: Option<String>,
}

#[cw_serde]
pub struct ContactsResponse {
    pub contacts: Vec<ContactResponse>,
}
