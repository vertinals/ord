use std::sync::Arc;
use redb::{ReadOnlyTable, RedbKey, RedbValue, Table, TableDefinition};
use serde::{Deserialize, Serialize};
use crate::{Index};
use crate::okx::datastore::cache::CacheTableIndex;

// pub enum TraceOperation {
//     Update(Vec<u8>),
//     Delete(Vec<u8>),
//     Insert,
// }

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

#[derive(Clone)]
pub struct IndexTracer {
    pub index: Arc<Index>,
    pub traces: Vec<TraceNode>,
}

pub fn string_to_bytes(s: &str) -> Vec<u8> {
    let byte_slice: &[u8] = s.as_bytes();
    let byte_vec: Vec<u8> = byte_slice.to_vec();
    byte_vec
}

impl IndexTracer {
    pub fn use_read_table<K: RedbKey + 'static, V: RedbValue + 'static, T>(&self,
                                                                           table_def: TableDefinition<K, V>,
                                                                           f: impl FnOnce(ReadOnlyTable<K, V>) -> crate::Result<T>) -> crate::Result<T> {
        let rtx = self.index.begin_read()?;
        let table = rtx.0.open_table(table_def)?;
        let ret = f(table);
        ret
    }
    pub fn use_write_table<K: RedbKey + 'static, V: RedbValue + 'static, T>(&mut self,
                                                                            table_def: TableDefinition<K, V>,
                                                                            mut f: impl FnMut(Table<K, V>) -> crate::Result<(T, TraceNode)>) -> crate::Result<T> {
        let rtx = self.index.begin_write()?;
        let table = rtx.open_table(table_def)?;
        let (ret, node) = f(table)?;
        self.traces.push(node);
        Ok(ret)
    }
}