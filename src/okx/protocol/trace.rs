use serde::{Deserialize, Serialize};
use crate::okx::datastore::cache::CacheTableIndex;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BalanceDelta {
    pub origin_overall_balance_delta: u128,
    pub origin_transferable_balance_delta: u128,
    pub new_overall_balance_delta: u128,
    pub new_transferable_balance_delta: u128,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MintTokenInfoDelta {
    pub origin_minted: u128,
    pub new_minted: u128,
    pub new_latest_mint_number: u32,
}

#[derive(Clone,Serialize,Deserialize)]
pub struct TraceNode {
    pub trace_type: CacheTableIndex,
    // pub operation: TraceOperation,
    pub key: Vec<u8>,
}
