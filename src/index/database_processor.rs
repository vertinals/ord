use std::sync::Arc;
use bitcoin::{OutPoint, TxOut};
use redb::{ReadTransaction, WriteTransaction};
use crate::{Index, InscriptionId, SatPoint};
use crate::index::{InscriptionEntryValue, InscriptionIdValue, TxidValue};

pub struct DataBaseProcessor<'a, 'db> {
    rtx:&'a ReadTransaction<'db>,
    wtx: &'a WriteTransaction<'db>,
}

impl<'a, 'db> DataBaseProcessor<'a, 'db> {
    pub(crate) fn tx_out_cache_insert(&self, p0: &OutPoint, p1: TxOut) -> crate::Result<()> {
        todo!()
    }
}

impl<'a, 'db> DataBaseProcessor<'a, 'db> {
    pub(crate) fn transaction_id_to_transaction_insert(&self, tx_id: &TxidValue, value: &[u8]) -> crate::Result<()> {
        todo!()
    }
}

impl<'a, 'db> DataBaseProcessor<'a, 'db> {
    pub fn sequence_number_to_entry_get(&self, initial_inscription_sequence_number: u32) -> crate::Result<Option<InscriptionEntryValue>> {
        todo!()
    }
}

impl<'a, 'db> DataBaseProcessor<'a, 'db> {
    pub(crate) fn id_to_sequence_number_get(&self, x: InscriptionIdValue) -> crate::Result<Option<u32>> {
        todo!()
    }
}

impl<'a, 'db> DataBaseProcessor<'a, 'db> {
    pub(crate) fn get_txout_by_outpoint(&self, p0: &OutPoint) -> crate::Result<u64> {
        todo!()
    }
}

impl<'a, 'db> DataBaseProcessor<'a, 'db> {
    pub fn inscriptions_on_output(&self, prev_output: &OutPoint) -> crate::Result<Vec<(SatPoint, InscriptionId)>> {
        // let ret = Index::inscriptions_on_output(
        // self.satpoint_to_sequence_number,
        // self.sequence_number_to_entry,
        // prev_output.clone())?;
        // TODO: twice
        todo!()
    }
}
