use cw_storage_plus::Item;

pub const PREVIOUS_BLOCK_TIME: Item<u64> = Item::new("prev-block-time");
