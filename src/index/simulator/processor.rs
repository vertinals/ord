use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use anyhow::anyhow;
use bitcoin::{OutPoint, Transaction, Txid, TxOut};
use indexer_sdk::client::drect::DirectClient;
use indexer_sdk::client::SyncClient;
use indexer_sdk::storage::db::memory::MemoryDB;
use indexer_sdk::storage::db::thread_safe::ThreadSafeDB;
use indexer_sdk::storage::kv::KVStorageProcessor;
use redb::{MultimapTable, ReadableTable, ReadOnlyTable, RedbKey, RedbValue, Table, TableDefinition};
use crate::{Index, InscriptionId, SatPoint};
use crate::index::entry::{SatPointValue};
use crate::index::{HOME_INSCRIPTIONS, INSCRIPTION_ID_TO_SEQUENCE_NUMBER, InscriptionEntryValue, InscriptionIdValue, OUTPOINT_TO_ENTRY, OutPointValue, SATPOINT_TO_SEQUENCE_NUMBER, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, Statistic, STATISTIC_TO_COUNT, TxidValue};
use crate::okx::datastore::ord::redb::table::get_txout_by_outpoint;
use crate::okx::protocol::context::Context;
use crate::okx::protocol::simulate::SimulateContext;
use crate::okx::protocol::trace::TraceNode;

#[derive(Clone)]
pub struct IndexWrapper {
    pub internal: Arc<Index>,
}

impl IndexWrapper {
    pub fn use_internal_table<K: RedbKey + 'static, V: RedbValue + 'static, T>(&self,
                                                                               table_def: TableDefinition<K, V>,
                                                                               f: impl FnOnce(ReadOnlyTable<K, V>) -> crate::Result<T>) -> crate::Result<T> {
        let rtx = self.internal.begin_read()?;
        let table = rtx.0.open_table(table_def)?;
        let ret = f(table);
        ret
    }
    pub fn new(internal: Arc<Index>) -> Self {
        Self { internal }
    }
}

// could be trait
#[derive(Clone)]
pub struct StorageProcessor<'a, 'db, 'tx> {
    pub internal: IndexWrapper,

    pub(super) home_inscriptions: Rc<RefCell<Table<'db, 'tx, u32, InscriptionIdValue>>>,
    pub(super) id_to_sequence_number: Rc<RefCell<Table<'db, 'tx, InscriptionIdValue, u32>>>,
    pub(super) inscription_number_to_sequence_number: Rc<RefCell<Table<'db, 'tx, i32, u32>>>,
    pub(super) outpoint_to_entry: Rc<RefCell<Table<'db, 'tx, &'static OutPointValue, &'static [u8]>>>,
    pub(super) transaction_id_to_transaction:
    Rc<RefCell<Table<'db, 'tx, &'static TxidValue, &'static [u8]>>>,
    pub(super) sat_to_sequence_number: Rc<RefCell<MultimapTable<'db, 'tx, u64, u32>>>,
    pub(super) satpoint_to_sequence_number:
    Rc<RefCell<MultimapTable<'db, 'tx, &'static SatPointValue, u32>>>,
    pub(super) sequence_number_to_children: Rc<RefCell<MultimapTable<'db, 'tx, u32, u32>>>,
    pub(super) sequence_number_to_satpoint: Rc<RefCell<Table<'db, 'tx, u32, &'static SatPointValue>>>,
    pub(super) sequence_number_to_inscription_entry: Rc<RefCell<Table<'db, 'tx, u32, InscriptionEntryValue>>>,

    pub outpoint_to_sat_ranges: Rc<RefCell<Table<'db, 'tx, &'static OutPointValue, &'static [u8]>>>,
    pub sat_to_satpoint: Rc<RefCell<Table<'db, 'tx, u64, &'static SatPointValue>>>,

    pub statistic_to_count: Rc<RefCell<Table<'db, 'tx, u64, u64>>>,
    pub _marker_a: PhantomData<&'a ()>,

    pub client: Option<DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>>,

    pub traces: Rc<RefCell<Vec<TraceNode>>>,
    pub context: SimulateContext<'a, 'db, 'tx>,
}

unsafe impl<'a, 'db, 'tx> Send for StorageProcessor<'a, 'db, 'tx> {}

unsafe impl<'a, 'db, 'tx> Sync for StorageProcessor<'a, 'db, 'tx> {}


impl<'a, 'db, 'tx> StorageProcessor<'a, 'db, 'tx> {
    pub fn create_context(&self) -> crate::Result<Context> {
        todo!()
    }
    pub fn get_transaction(&self, tx_id: &Txid) -> crate::Result<Option<Transaction>> {
        let client = self.client.as_ref().unwrap();
        let ret = client.get_transaction_by_tx_id(tx_id.clone())?;
        Ok(ret)
    }
    pub fn create_simulate_context(&self) -> crate::Result<SimulateContext<'a, 'db, 'tx>> {
        Ok(self.context.clone())
    }
    pub fn next_sequence_number(&self) -> crate::Result<u32> {
        let table = self.sequence_number_to_inscription_entry.borrow();
        let ret: u32 = table
            .iter()?
            .next_back()
            .and_then(|result| result.ok())
            .map(|(number, _id)| number.value() + 1)
            .unwrap_or(0);
        let v = self.internal.use_internal_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, |table| Ok({
            table
                .iter()?
                .next_back()
                .and_then(|result| result.ok())
                .map(|(number, _id)| number.value() + 1)
                .unwrap_or(0)
        }))?;
        if ret > v {
            return Ok(ret);
        }
        Ok(v)
    }
    pub fn id_to_sequence_number_get(&self, x: &InscriptionIdValue) -> crate::Result<Option<u32>> {
        let table = self.id_to_sequence_number.borrow();
        let v = table.get(x)?;
        if let Some(v) = v {
            return Ok(Some(v.value()));
        }
        let ret = self.internal.use_internal_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER, |table| {
            let value = table.get(x).map_err(|e| {
                anyhow!("id_to_sequence_number_get error:{}",e)
            })?;
            if let Some(value) = value {
                return Ok(Some(value.value()));
            }
            return Ok(None);
        })?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        Ok(None)
    }
    pub fn sequence_number_to_entry_get(&self, initial_inscription_sequence_number: u32) -> crate::Result<Option<InscriptionEntryValue>> {
        let table = self.sequence_number_to_inscription_entry.borrow();
        let value = table.get(initial_inscription_sequence_number)?;
        if let Some(v) = value {
            return Ok(Some(v.value()));
        }
        let ret = self.internal.use_internal_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, |table| {
            let ret = table.get(initial_inscription_sequence_number).map_err(move |e| {
                anyhow!("sequence_number_to_entry_get error:{}",e)
            })?;
            if let Some(ret) = ret {
                return Ok(Some(ret.value()));
            }
            return Ok(None);
        })?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        Ok(None)
    }
    pub fn get_lost_sats(&self) -> crate::Result<u64> {
        let table = self.statistic_to_count.borrow();
        let ret = table
            .get(&Statistic::LostSats.key())?
            .map(|lost_sats| lost_sats.value())
            .unwrap_or(0);
        if ret == 0 {
            let ret = self.internal.use_internal_table(STATISTIC_TO_COUNT, |table| Ok({
                table
                    .get(&Statistic::LostSats.key())?
                    .map(|lost_sats| lost_sats.value())
                    .unwrap_or(0)
            }))?;
            return Ok(ret);
        }
        return Ok(ret);
    }
    pub fn get_cursed_inscription_count(&self) -> crate::Result<u64> {
        let table = self.statistic_to_count.borrow();
        let ret = table
            .get(&Statistic::CursedInscriptions.key())?
            .map(|count| count.value())
            .unwrap_or(0);
        if ret != 0 {
            return Ok(ret);
        }
        let ret = self.internal.use_internal_table(STATISTIC_TO_COUNT, |table| Ok({
            table
                .get(&Statistic::CursedInscriptions.key())?
                .map(|count| count.value())
                .unwrap_or(0)
        }))?;
        return Ok(ret);
    }
    pub fn get_blessed_inscription_count(&self) -> crate::Result<u64> {
        let table = self.statistic_to_count.borrow();
        let ret = table
            .get(&Statistic::BlessedInscriptions.key())?
            .map(|count| count.value())
            .unwrap_or(0);
        if ret != 0 {
            return Ok(ret);
        }
        let ret = self.internal.use_internal_table(STATISTIC_TO_COUNT, |table| Ok({
            table
                .get(&Statistic::BlessedInscriptions.key())?
                .map(|count| count.value())
                .unwrap_or(0)
        }))?;
        Ok(ret)
    }
    pub fn get_unbound_inscriptions(&self) -> crate::Result<u64> {
        let table = self.statistic_to_count.borrow();
        let ret = table
            .get(&Statistic::UnboundInscriptions.key())?
            .map(|unbound_inscriptions| unbound_inscriptions.value())
            .unwrap_or(0);
        if ret > 0 {
            return Ok(ret);
        }
        let ret = self.internal.use_internal_table(STATISTIC_TO_COUNT, |table| Ok({
            table
                .get(&Statistic::UnboundInscriptions.key())?
                .map(|count| count.value())
                .unwrap_or(0)
        }))?;
        Ok(ret)
    }
    pub fn get_txout_by_outpoint(&self, x: &OutPoint) -> crate::Result<Option<TxOut>> {
        let bindind = self.outpoint_to_entry.borrow();
        let ret = get_txout_by_outpoint(bindind.deref(), x)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        let ret = self.internal.use_internal_table(OUTPOINT_TO_ENTRY, |table| {
            get_txout_by_outpoint(&table, x)
        })?;
        Ok(ret)
    }
    pub fn inscriptions_on_output(&self, prev_output: &OutPoint) -> crate::Result<Vec<(SatPoint, InscriptionId)>> {
        let table = self.satpoint_to_sequence_number.borrow();
        let satpoint_to_sequence_number = table.deref();

        let table = self.sequence_number_to_inscription_entry.borrow();
        let sequence_number_to_entry = table.deref();
        let ret = Index::inscriptions_on_output(
            satpoint_to_sequence_number,
            sequence_number_to_entry,
            prev_output.clone())?;

        let mut set: HashSet<(SatPoint, InscriptionId)> = ret.into_iter()
            .map(|(k, v)| {
                (k, v)
            }).collect();

        let rtx = self.internal.internal.begin_read()?;
        let satpoint_to_sequence_number = rtx.0.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
        let sequence_number_to_entry = rtx.0.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
        let ret = Index::inscriptions_on_output(
            &satpoint_to_sequence_number,
            &sequence_number_to_entry,
            prev_output.clone())?;
        for node in ret {
            if set.contains(&node) {
                continue;
            }
            set.insert(node);
        }
        Ok(set.into_iter().collect())
    }
    pub fn home_inscriptions_len(&self) -> u64 {
        let table = self.home_inscriptions.borrow();
        let sim = table.len().unwrap();
        let ret = self.internal.use_internal_table(HOME_INSCRIPTIONS, |table| {
            // TODO
            Ok(table.len().unwrap())
        }).unwrap();
        return sim + ret;
    }
    pub fn sequence_number_to_satpoint_insert(&self, sequence_number: u32, sat_point: &SatPointValue) -> crate::Result<()> {
        let mut table = self.sequence_number_to_satpoint.borrow_mut();
        table.insert(sequence_number, sat_point)?;

        // let key = u32::as_bytes(&sequence_number).to_vec();
        // let value = sat_point.to_vec();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_SATPOINT, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        Ok(())
    }
    pub fn satpoint_to_sequence_number_insert(&self, sat_point: &SatPointValue, sequence: u32) -> crate::Result<()> {
        let mut table = self.sequence_number_to_satpoint.borrow_mut();
        table.insert(sequence, sat_point)?;

        // let key = sat_point.to_vec();
        // let value = u32::as_bytes(&sequence).to_vec();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        Ok(())
    }
    pub fn home_inscriptions_pop_first(&self) -> crate::Result<()> {
        let mut table = self.home_inscriptions.borrow_mut();
        table.pop_first()?;

        // self.cache_writer.use_cache_mut(CacheTableIndex::HOME_INSCRIPTIONS, |v| {
        //     v.pop_first()
        // });
        Ok(())
    }
    pub fn home_inscriptions_insert(&self, sequence_number: &u32, value: InscriptionIdValue) -> crate::Result<()> {
        // let key = u32::as_bytes(sequence_number).to_vec();
        // let value = InscriptionIdValue::as_bytes(&value).to_vec();
        // self.cache_writer.use_cache_mut(CacheTableIndex::HOME_INSCRIPTIONS, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())

        let mut table = self.home_inscriptions.borrow_mut();
        table
            .insert(sequence_number, value)?;
        Ok(())
    }
    pub fn id_to_sequence_number_insert(&self, value: &InscriptionIdValue, sequence_number: u32) -> crate::Result<()> {
        // let key = rmp_serde::to_vec(value).unwrap();
        // let value = sequence.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_ID_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        let mut table = self.id_to_sequence_number.borrow_mut();
        table
            .insert(value, sequence_number)?;
        Ok(())
    }
    pub fn sequence_number_to_children_insert(&self, parent_sequence_number: u32, sequence_number: u32) -> crate::Result<()> {
        // let key = sequence.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(value).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_CHILDREN, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        let mut table = self.sequence_number_to_children.borrow_mut();
        table.insert(parent_sequence_number, sequence_number)?;
        Ok(())
    }
    pub fn sequence_number_to_entry_insert(&self, sequence: u32, value: &InscriptionEntryValue) -> crate::Result<()> {
        // let key = sequence.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(value).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        let mut table = self.sequence_number_to_inscription_entry.borrow_mut();
        table.insert(sequence, value)?;
        Ok(())
    }
    pub fn sat_to_sequence_number_insert(&self, n: &u64, sequence_number: &u32) -> crate::Result<()> {
        // let key = n.to_le_bytes().as_slice();
        // let value = sequence.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        let mut table = self.sat_to_sequence_number.borrow_mut();
        table.insert(n, sequence_number)?;
        Ok(())
    }
    pub fn inscription_number_to_sequence_number_insert(&self, inscription_number: i32, sequence_number: u32) -> crate::Result<()> {
        // let key = inscription_number.to_le_bytes().as_slice();
        // let value = sequence_number.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        let mut table = self.inscription_number_to_sequence_number.borrow_mut();
        table.insert(inscription_number, sequence_number)?;
        Ok(())
    }
    pub fn outpoint_to_entry_insert(&self, value: &OutPointValue, entry: &[u8]) -> crate::Result<()> {
        let mut table = self.outpoint_to_entry.borrow_mut();
        table.insert(value, entry)?;
        Ok(())
        // let key = rmp_serde::to_vec(value).unwrap();
        // let value = entry.to_vec();
        // self.cache_writer.use_cache_mut(CacheTableIndex::OUTPOINT_TO_ENTRY, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
    }
    pub fn transaction_id_to_transaction_insert(&self, tx_id: &TxidValue, value: &[u8]) -> crate::Result<()> {
        let mut table = self.transaction_id_to_transaction.borrow_mut();
        table.insert(tx_id, value)?;
        Ok(())
    }
    pub fn outpoint_to_sat_ranges_insert(&self, value: &OutPointValue, data: &[u8]) -> crate::Result<()> {
        let mut table = self.outpoint_to_sat_ranges.borrow_mut();
        table.insert(value, data)?;
        Ok(())
    }
    pub fn outpoint_to_sat_ranges_remove(&self, k: &OutPointValue) -> crate::Result<Option<Vec<u8>>> {
        let mut table = self.outpoint_to_sat_ranges.borrow_mut();
        let ret: Vec<u8> = table.remove(k)?.map(|ranges| ranges.value().to_vec()).unwrap_or_default();
        return Ok(Some(ret));
    }
    pub fn satpoint_to_sequence_number_remove_all(&self, v: &SatPointValue) -> crate::Result<()> {
        let mut table = self.satpoint_to_sequence_number.borrow_mut();
        table
            .remove_all(v)?;
        Ok(())
    }
    pub fn sat_to_satpoint_insert(&self, key: &u64, value: &SatPointValue) -> crate::Result<()> {
        let mut table = self.sat_to_satpoint.borrow_mut();
        table.insert(key, value)?;
        Ok(())
    }
}