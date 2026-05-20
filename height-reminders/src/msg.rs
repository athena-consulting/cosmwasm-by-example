use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::Reminder;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    CreateReminder {
        id: String,
        due_height: u64,
        note: String,
    },
    CompleteReminder {
        id: String,
    },
    CancelReminder {
        id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ReminderResponse)]
    Reminder { id: String },
    #[returns(ListRemindersResponse)]
    ListReminders {},
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ReminderResponse {
    pub reminder: Reminder,
}

#[cw_serde]
pub struct ListRemindersResponse {
    pub ids: Vec<String>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
}
