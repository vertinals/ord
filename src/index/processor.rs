// use std::sync::Arc;
// use bitcoin::{OutPoint, TxOut};
// use redb::{RedbValue, WriteTransaction};
// use crate::{Index, InscriptionId, SatPoint};
// use crate::index::entry::SatPointValue;
// use crate::index::{InscriptionEntryValue, InscriptionIdValue, OutPointValue, TxidValue};
// use crate::okx::datastore::cache::{CacheTableIndex, CacheWriter};
// use crate::okx::protocol::context::Context;
//
// pub struct StorageProcessor<'a, 'db,'tx> {
//     pub cache_writer: CacheWriter,
//     pub internal: Arc<Index>,
//
//     pub wtx: &'a mut WriteTransaction<'db>,
// }
//
// unsafe impl<'a, 'db,'tx> Send for StorageProcessor<'a, 'db,'tx> {}
//
// unsafe impl<'a, 'db,'tx> Sync for StorageProcessor<'a, 'db,'tx> {}
//
//
// impl<'a, 'db,'tx> StorageProcessor<'a, 'db,'tx> {
//     pub(crate) fn create_context(&self) -> crate::Result<Context> {
//         todo!()
//     }
//     pub(crate) fn next_sequence_number(&self) -> crate::Result<u32> {
//         todo!()
//     }
//     pub(crate) fn outpoint_to_sat_ranges_insert(&self, value: &OutPointValue, data: &[u8]) -> crate::Result<()> {
//         todo!()
//     }
//     pub(crate) fn outpoint_to_sat_ranges_remove(&self, p0: &OutPointValue) -> crate::Result<Option<Vec<u8>>> {
//         todo!()
//     }
//     pub fn get_lost_sats(&self) -> crate::Result<u64> {
//         todo!()
//     }
//     pub fn get_cursed_inscription_count(&self) -> crate::Result<u64> {
//         todo!()
//     }
//     pub fn get_blessed_inscription_count(&self) -> crate::Result<u64> {
//         todo!()
//     }
//     pub fn get_unbound_inscriptions(&self) -> crate::Result<u64> {
//         todo!()
//     }
//     pub fn get_next_sequence_number(&self) -> crate::Result<u64> {
//         todo!()
//     }
//     pub fn sat_to_satpoint_insert(&self, key: &u64, value: &SatPointValue) -> crate::Result<()> {
//         todo!()
//     }
//     pub fn get_txout_by_outpoint(&self, x: &OutPoint) -> crate::Result<Option<TxOut>> {
//         todo!()
//     }
//     pub(crate) fn satpoint_to_sequence_number_remove_all(&self, v: &SatPointValue) -> crate::Result<()> {
//         // self
//         //     .satpoint_to_sequence_number
//         //     .remove_all(v)?;
//         // Ok(())
//         todo!()
//     }
//     pub(crate) fn home_inscriptions_len(&self) -> u64 {
//         todo!()
//     }
//     pub(crate) fn sequence_number_to_satpoint_insert(&self, sequence_number: u32, sat_point: &SatPointValue) -> crate::Result<()> {
//         // self.sequence_number_to_satpoint.insert(sequence_number, sat_point)?;
//
//         let key = u32::as_bytes(&sequence_number).to_vec();
//         let value = sat_point.to_vec();
//         self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_SATPOINT, |v| {
//             v.insert(key.to_vec(), value.to_vec());
//         });
//         Ok(())
//     }
//     pub(crate) fn satpoint_to_sequence_number_insert(&self, sat_point: &SatPointValue, sequence: u32) -> crate::Result<()> {
//         // self.sequence_number_to_satpoint.insert(sequence, sat_point)?;
//
//         let key = sat_point.to_vec();
//         let value = u32::as_bytes(&sequence).to_vec();
//         self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
//             v.insert(key.to_vec(), value.to_vec());
//         });
//         Ok(())
//     }
//     pub(crate) fn home_inscriptions_pop_first(&self) -> crate::Result<()> {
//         // self.home_inscriptions.pop_first()?;
//
//         self.cache_writer.use_cache_mut(CacheTableIndex::HOME_INSCRIPTIONS, |v| {
//             v.pop_first()
//         });
//         Ok(())
//     }
//     pub(crate) fn home_inscriptions_insert(&self, sequence_number: &u32, value: InscriptionIdValue) -> crate::Result<()> {
//         let key = u32::as_bytes(sequence_number).to_vec();
//         let value = InscriptionIdValue::as_bytes(&value).to_vec();
//         self.cache_writer.use_cache_mut(CacheTableIndex::HOME_INSCRIPTIONS, |v| {
//             v.insert(key.to_vec(), value.to_vec());
//         });
//         Ok(())
//         // self
//         //     .home_inscriptions
//         //     .insert(sequence_number, value)?;
//         // Ok(())
//     }
//     pub(crate) fn id_to_sequence_number_insert(&self, value: &InscriptionIdValue, sequence_number: u32) -> crate::Result<()> {
//         // let key = rmp_serde::to_vec(value).unwrap();
//         // let value = sequence.to_le_bytes().as_slice();
//         // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_ID_TO_SEQUENCE_NUMBER, |v| {
//         //     v.insert(key.to_vec(), value.to_vec());
//         // });
//         // Ok(())
//         // self
//         //     .id_to_sequence_number
//         //     .insert(value, sequence_number)?;
//         Ok(())
//     }
//     pub(crate) fn sequence_number_to_children_insert(&self, parent_sequence_number: u32, sequence_number: u32) -> crate::Result<()> {
//         // let key = sequence.to_le_bytes().as_slice();
//         // let value = rmp_serde::to_vec(value).unwrap();
//         // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_CHILDREN, |v| {
//         //     v.insert(key.to_vec(), value.to_vec());
//         // });
//         // Ok(())
//         // self
//         //     .sequence_number_to_children
//         //     .insert(parent_sequence_number, sequence_number)?;
//         Ok(())
//     }
//     pub(crate) fn sequence_number_to_entry_insert(&self, sequence: u32, value: &InscriptionEntryValue) -> crate::Result<()> {
//         // let key = sequence.to_le_bytes().as_slice();
//         // let value = rmp_serde::to_vec(value).unwrap();
//         // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, |v| {
//         //     v.insert(key.to_vec(), value.to_vec());
//         // });
//         // Ok(())
//         // self.sequence_number_to_entry.insert(sequence, value)?;
//         Ok(())
//     }
//     pub(crate) fn sat_to_sequence_number_insert(&self, n: &u64, sequence_number: &u32) -> crate::Result<()> {
//         // let key = n.to_le_bytes().as_slice();
//         // let value = sequence.to_le_bytes().as_slice();
//         // self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
//         //     v.insert(key.to_vec(), value.to_vec());
//         // });
//         // Ok(())
//         // self.sat_to_sequence_number.insert(n, sequence_number)?;
//         Ok(())
//     }
//     pub(crate) fn inscription_number_to_sequence_number_insert(&self, inscription_number: i32, sequence_number: u32) -> crate::Result<()> {
//         // let key = inscription_number.to_le_bytes().as_slice();
//         // let value = sequence_number.to_le_bytes().as_slice();
//         // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, |v| {
//         //     v.insert(key.to_vec(), value.to_vec());
//         // });
//         // Ok(())
//         // self
//         // .inscription_number_to_sequence_number
//         // .insert(inscription_number, sequence_number)?;
//         Ok(())
//     }
//     pub(crate) fn outpoint_to_entry_insert(&self, value: &OutPointValue, entry: &[u8]) -> crate::Result<()> {
//         // self.outpoint_to_entry.insert(value, entry)?;
//         Ok(())
//         // let key = rmp_serde::to_vec(value).unwrap();
//         // let value = entry.to_vec();
//         // self.cache_writer.use_cache_mut(CacheTableIndex::OUTPOINT_TO_ENTRY, |v| {
//         //     v.insert(key.to_vec(), value.to_vec());
//         // });
//         // Ok(())
//     }
//     pub fn inscriptions_on_output(&self, prev_output: &OutPoint) -> crate::Result<Vec<(SatPoint, InscriptionId)>> {
//         // let ret = Index::inscriptions_on_output(
//         // self.satpoint_to_sequence_number,
//         // self.sequence_number_to_entry,
//         // prev_output.clone())?;
//         // TODO: twice
//         todo!()
//     }
//
//     pub(crate) fn transaction_id_to_transaction_insert(&self, tx_id: &TxidValue, value: &[u8]) -> crate::Result<()> {
//         // self
//         //     .transaction_id_to_transaction
//         //     .insert(tx_id, value)?;
//
//         Ok(())
//     }
//
//     pub(crate) fn id_to_sequence_number_get(&self, x: InscriptionIdValue) -> crate::Result<Option<u32>> {
//         // TODO,twice
//         // let ret = self.id_to_sequence_number.get(x)?.unwrap().value();
//         // Ok(Some(ret))
//         todo!()
//     }
//     pub fn sequence_number_to_entry_get(&self, initial_inscription_sequence_number: u32) -> crate::Result<Option<InscriptionEntryValue>> {
//         // TODO: twice
//         // let ret = self
//         //     .sequence_number_to_entry
//         //     .get(initial_inscription_sequence_number)?
//         //     .unwrap()
//         //     .value();
//         // Ok(Some(ret))
//         todo!()
//     }
// }
//////////////////


use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use bitcoin::{OutPoint, TxOut};
use redb::{MultimapTable, RedbValue, Table, WriteTransaction};
use crate::{Index, InscriptionId, SatPoint};
use crate::index::entry::SatPointValue;
use crate::index::{InscriptionEntryValue, InscriptionIdValue, OutPointValue, TxidValue};
use crate::okx::datastore::cache::{CacheTableIndex, CacheWriter};
use crate::okx::protocol::context::Context;
use crate::okx::protocol::simulate::SimulateContext;

#[derive(Clone)]
pub struct StorageProcessor<'a, 'db, 'tx> {
    pub internal: Arc<Index>,

    // pub wtx: &'a mut WriteTransaction<'db>,

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
    pub(super) sequence_number_to_entry: Rc<RefCell<Table<'db, 'tx, u32, InscriptionEntryValue>>>,
    pub(super) sequence_number_to_satpoint: Rc<RefCell<Table<'db, 'tx, u32, &'static SatPointValue>>>,
    pub _marker_a: PhantomData<&'a ()>,
}

unsafe impl<'a, 'db, 'tx> Send for StorageProcessor<'a, 'db, 'tx> {}

unsafe impl<'a, 'db, 'tx> Sync for StorageProcessor<'a, 'db, 'tx> {}


impl<'a, 'db, 'tx> StorageProcessor<'a, 'db, 'tx> {
    pub(crate) fn create_context(&self) -> crate::Result<Context> {
        todo!()
    }
    pub(crate) fn create_simulate_context(&self) -> crate::Result<SimulateContext> {
        todo!()
    }
    pub(crate) fn next_sequence_number(&self) -> crate::Result<u32> {
        todo!()
    }
    pub(crate) fn outpoint_to_sat_ranges_insert(&self, value: &OutPointValue, data: &[u8]) -> crate::Result<()> {
        todo!()
    }
    pub(crate) fn outpoint_to_sat_ranges_remove(&self, p0: &OutPointValue) -> crate::Result<Option<Vec<u8>>> {
        todo!()
    }
    pub fn get_lost_sats(&self) -> crate::Result<u64> {
        todo!()
    }
    pub fn get_cursed_inscription_count(&self) -> crate::Result<u64> {
        todo!()
    }
    pub fn get_blessed_inscription_count(&self) -> crate::Result<u64> {
        todo!()
    }
    pub fn get_unbound_inscriptions(&self) -> crate::Result<u64> {
        todo!()
    }
    pub fn get_next_sequence_number(&self) -> crate::Result<u64> {
        todo!()
    }
    pub fn sat_to_satpoint_insert(&self, key: &u64, value: &SatPointValue) -> crate::Result<()> {
        todo!()
    }
    pub fn get_txout_by_outpoint(&self, x: &OutPoint) -> crate::Result<Option<TxOut>> {
        todo!()
    }
    pub(crate) fn satpoint_to_sequence_number_remove_all(&self, v: &SatPointValue) -> crate::Result<()> {
        // self
        //     .satpoint_to_sequence_number
        //     .remove_all(v)?;
        // Ok(())
        todo!()
    }
    pub(crate) fn home_inscriptions_len(&self) -> u64 {
        todo!()
    }
    pub(crate) fn sequence_number_to_satpoint_insert(&self, sequence_number: u32, sat_point: &SatPointValue) -> crate::Result<()> {
        let mut table = self.sequence_number_to_satpoint.borrow_mut();
        table.insert(sequence_number, sat_point)?;

        // let key = u32::as_bytes(&sequence_number).to_vec();
        // let value = sat_point.to_vec();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_SATPOINT, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        Ok(())
    }
    pub(crate) fn satpoint_to_sequence_number_insert(&self, sat_point: &SatPointValue, sequence: u32) -> crate::Result<()> {
        let mut table = self.sequence_number_to_satpoint.borrow_mut();
        table.insert(sequence, sat_point)?;

        // let key = sat_point.to_vec();
        // let value = u32::as_bytes(&sequence).to_vec();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        Ok(())
    }
    pub(crate) fn home_inscriptions_pop_first(&self) -> crate::Result<()> {
        let mut table = self.home_inscriptions.borrow_mut();
        table.pop_first()?;

        // self.cache_writer.use_cache_mut(CacheTableIndex::HOME_INSCRIPTIONS, |v| {
        //     v.pop_first()
        // });
        Ok(())
    }
    pub(crate) fn home_inscriptions_insert(&self, sequence_number: &u32, value: InscriptionIdValue) -> crate::Result<()> {
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
    pub(crate) fn id_to_sequence_number_insert(&self, value: &InscriptionIdValue, sequence_number: u32) -> crate::Result<()> {
        // let key = rmp_serde::to_vec(value).unwrap();
        // let value = sequence.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_ID_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        // self
        //     .id_to_sequence_number
        //     .insert(value, sequence_number)?;
        Ok(())
    }
    pub(crate) fn sequence_number_to_children_insert(&self, parent_sequence_number: u32, sequence_number: u32) -> crate::Result<()> {
        // let key = sequence.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(value).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_CHILDREN, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        // self
        //     .sequence_number_to_children
        //     .insert(parent_sequence_number, sequence_number)?;
        Ok(())
    }
    pub(crate) fn sequence_number_to_entry_insert(&self, sequence: u32, value: &InscriptionEntryValue) -> crate::Result<()> {
        // let key = sequence.to_le_bytes().as_slice();
        // let value = rmp_serde::to_vec(value).unwrap();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        // self.sequence_number_to_entry.insert(sequence, value)?;
        Ok(())
    }
    pub(crate) fn sat_to_sequence_number_insert(&self, n: &u64, sequence_number: &u32) -> crate::Result<()> {
        // let key = n.to_le_bytes().as_slice();
        // let value = sequence.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::SAT_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        // self.sat_to_sequence_number.insert(n, sequence_number)?;
        Ok(())
    }
    pub(crate) fn inscription_number_to_sequence_number_insert(&self, inscription_number: i32, sequence_number: u32) -> crate::Result<()> {
        // let key = inscription_number.to_le_bytes().as_slice();
        // let value = sequence_number.to_le_bytes().as_slice();
        // self.cache_writer.use_cache_mut(CacheTableIndex::INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, |v| {
        //     v.insert(key.to_vec(), value.to_vec());
        // });
        // Ok(())
        // self
        // .inscription_number_to_sequence_number
        // .insert(inscription_number, sequence_number)?;
        Ok(())
    }
    pub(crate) fn outpoint_to_entry_insert(&self, value: &OutPointValue, entry: &[u8]) -> crate::Result<()> {
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
    pub fn inscriptions_on_output(&self, prev_output: &OutPoint) -> crate::Result<Vec<(SatPoint, InscriptionId)>> {
        // let ret = Index::inscriptions_on_output(
        // self.satpoint_to_sequence_number,
        // self.sequence_number_to_entry,
        // prev_output.clone())?;
        // TODO: twice
        todo!()
    }

    pub(crate) fn transaction_id_to_transaction_insert(&self, tx_id: &TxidValue, value: &[u8]) -> crate::Result<()> {
        // self
        //     .transaction_id_to_transaction
        //     .insert(tx_id, value)?;

        Ok(())
    }

    pub(crate) fn id_to_sequence_number_get(&self, x: InscriptionIdValue) -> crate::Result<Option<u32>> {
        // TODO,twice
        // let ret = self.id_to_sequence_number.get(x)?.unwrap().value();
        // Ok(Some(ret))
        todo!()
    }
    pub fn sequence_number_to_entry_get(&self, initial_inscription_sequence_number: u32) -> crate::Result<Option<InscriptionEntryValue>> {
        // TODO: twice
        // let ret = self
        //     .sequence_number_to_entry
        //     .get(initial_inscription_sequence_number)?
        //     .unwrap()
        //     .value();
        // Ok(Some(ret))
        todo!()
    }
}