use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VoucherStatus {
    Open,
    Claimed,
    Refunded,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Voucher {
    pub creator: Addr,
    pub recipient: Addr,
    pub funds: Vec<Coin>,
    pub expires_at_height: u64,
    pub status: VoucherStatus,
}

pub const VOUCHERS: Map<&str, Voucher> = Map::new("vouchers");
