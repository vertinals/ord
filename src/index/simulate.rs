use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use anyhow::anyhow;
use bitcoin::{OutPoint, Transaction, Txid, TxOut};
use redb::{ReadableTable, Table, WriteTransaction};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::TryRecvError;
use crate::{Index, Options, Rune, RuneEntry, Sat, SatPoint, timestamp};
use crate::height::Height;
use crate::index::{BlockData, BRC20_BALANCES, BRC20_EVENTS, BRC20_INSCRIBE_TRANSFER, BRC20_TOKEN, BRC20_TRANSFERABLELOG, COLLECTIONS_INSCRIPTION_ID_TO_KINDS, COLLECTIONS_KEY_TO_INSCRIPTION_ID, HEIGHT_TO_BLOCK_HEADER, HEIGHT_TO_LAST_SEQUENCE_NUMBER, HOME_INSCRIPTIONS, INSCRIPTION_ID_TO_SEQUENCE_NUMBER, INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, ORD_TX_TO_OPERATIONS, OUTPOINT_TO_ENTRY, OUTPOINT_TO_RUNE_BALANCES, OUTPOINT_TO_SAT_RANGES, RUNE_ID_TO_RUNE_ENTRY, RUNE_TO_RUNE_ID, SAT_TO_SATPOINT, SAT_TO_SEQUENCE_NUMBER, SATPOINT_TO_SEQUENCE_NUMBER, SEQUENCE_NUMBER_TO_CHILDREN, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, SEQUENCE_NUMBER_TO_RUNE_ID, SEQUENCE_NUMBER_TO_SATPOINT, Statistic, STATISTIC_TO_COUNT, TRANSACTION_ID_TO_RUNE, TRANSACTION_ID_TO_TRANSACTION};
use crate::index::entry::{Entry, SatPointValue, SatRange};
use crate::index::processor::StorageProcessor;
use crate::index::updater::pending_updater::PendingUpdater;
use crate::okx::datastore::cache::CacheWriter;
use crate::okx::datastore::ord::InscriptionOp;
use crate::okx::datastore::ord::redb::table::get_txout_by_outpoint;
use crate::okx::lru::SimpleLru;
use crate::okx::protocol::{BlockContext, ProtocolConfig, ProtocolManager};
use crate::okx::protocol::context::Context;
use crate::okx::protocol::trace::IndexTracer;

pub struct Simulator<'a, 'db, 'tx> {
    pub options: Options,
    // pub simulate_index: IndexTracer,
    pub internal_index: Arc<Index>,
    _marker_a: PhantomData<&'a ()>,
    _marker_b: PhantomData<&'db ()>,
    _marker_tx: PhantomData<&'tx ()>,
}

impl<'a, 'db, 'tx> Simulator<'a, 'db, 'tx> {
    pub fn simulate_tx(&self) {}

    // fn index_block(
    //     &mut self,
    //     index: &Index,
    //     height: u32,
    //     outpoint_sender: &mut Sender<OutPoint>,
    //     tx_out_receiver: &mut Receiver<TxOut>,
    //     block: BlockData,
    //     tx_out_cache: &mut SimpleLru<OutPoint, TxOut>,
    //     wtx: &mut WriteTransaction,
    // ) -> crate::Result<()> {
    //     let start = Instant::now();
    //     let mut sat_ranges_written = 0;
    //     let mut outputs_in_block = 0;
    //
    //     // If value_receiver still has values something went wrong with the last block
    //     // Could be an assert, shouldn't recover from this and commit the last block
    //     let Err(TryRecvError::Empty) = tx_out_receiver.try_recv() else {
    //         return Err(anyhow!("Previous block did not consume all input values"));
    //     };
    //
    //     let mut outpoint_to_entry = wtx.open_table(OUTPOINT_TO_ENTRY)?;
    //
    //     let index_inscriptions = true;
    //
    //     let fetching_outputs_count = AtomicUsize::new(0);
    //     let total_outputs_count = AtomicUsize::new(0);
    //     let cache_outputs_count = AtomicUsize::new(0);
    //     let miss_outputs_count = AtomicUsize::new(0);
    //     let meet_outputs_count = AtomicUsize::new(0);
    //     if index_inscriptions {
    //         // Send all missing input outpoints to be fetched right away
    //         let txids = block
    //             .txdata
    //             .iter()
    //             .map(|(_, txid)| txid)
    //             .collect::<HashSet<_>>();
    //         use rayon::prelude::*;
    //         let tx_outs = block
    //             .txdata
    //             .par_iter()
    //             .flat_map(|(tx, _)| tx.input.par_iter())
    //             .filter_map(|input| {
    //                 total_outputs_count.fetch_add(1, Ordering::Relaxed);
    //                 let prev_output = input.previous_output;
    //                 // We don't need coinbase input value
    //                 if prev_output.is_null() {
    //                     None
    //                 } else if txids.contains(&prev_output.txid) {
    //                     meet_outputs_count.fetch_add(1, Ordering::Relaxed);
    //                     None
    //                 } else if tx_out_cache.contains(&prev_output) {
    //                     cache_outputs_count.fetch_add(1, Ordering::Relaxed);
    //                     None
    //                 } else if let Some(txout) =
    //                     get_txout_by_outpoint(&outpoint_to_entry, &prev_output).unwrap()
    //                 {
    //                     miss_outputs_count.fetch_add(1, Ordering::Relaxed);
    //                     Some((prev_output, Some(txout)))
    //                 } else {
    //                     fetching_outputs_count.fetch_add(1, Ordering::Relaxed);
    //                     Some((prev_output, None))
    //                 }
    //             })
    //             .collect::<Vec<_>>();
    //         for (out_point, value) in tx_outs.into_iter() {
    //             if let Some(tx_out) = value {
    //                 tx_out_cache.insert(out_point, tx_out);
    //             } else {
    //                 outpoint_sender.blocking_send(out_point).unwrap();
    //             }
    //         }
    //     }
    //
    //     let time = timestamp(block.header.time);
    //
    //     log::info!(
    //   "Block {} at {} with {} transactions, fetching previous outputs {}/{}…, {},{},{}, cost:{}ms",
    //   height,
    //   time,
    //   block.txdata.len(),
    //   fetching_outputs_count.load(Ordering::Relaxed),
    //   total_outputs_count.load(Ordering::Relaxed),
    //   miss_outputs_count.load(Ordering::Relaxed),
    //   meet_outputs_count.load(Ordering::Relaxed),
    //   cache_outputs_count.load(Ordering::Relaxed),
    //   start.elapsed().as_millis(),
    // );
    //
    //     let mut height_to_block_header = wtx.open_table(HEIGHT_TO_BLOCK_HEADER)?;
    //     let mut height_to_last_sequence_number = wtx.open_table(HEIGHT_TO_LAST_SEQUENCE_NUMBER)?;
    //     let mut home_inscriptions = wtx.open_table(HOME_INSCRIPTIONS)?;
    //     let mut inscription_id_to_sequence_number =
    //         wtx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
    //     let mut inscription_number_to_sequence_number =
    //         wtx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?;
    //     let mut sat_to_sequence_number = wtx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?;
    //     let mut satpoint_to_sequence_number = wtx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
    //     let mut sequence_number_to_children = wtx.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?;
    //     let mut sequence_number_to_inscription_entry =
    //         wtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
    //     let mut sequence_number_to_satpoint = wtx.open_table(SEQUENCE_NUMBER_TO_SATPOINT)?;
    //     let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
    //     let mut transaction_id_to_transaction = wtx.open_table(TRANSACTION_ID_TO_TRANSACTION)?;
    //
    //     let mut lost_sats = statistic_to_count
    //         .get(&Statistic::LostSats.key())?
    //         .map(|lost_sats| lost_sats.value())
    //         .unwrap_or(0);
    //
    //     let cursed_inscription_count = statistic_to_count
    //         .get(&Statistic::CursedInscriptions.key())?
    //         .map(|count| count.value())
    //         .unwrap_or(0);
    //
    //     let blessed_inscription_count = statistic_to_count
    //         .get(&Statistic::BlessedInscriptions.key())?
    //         .map(|count| count.value())
    //         .unwrap_or(0);
    //
    //     let unbound_inscriptions = statistic_to_count
    //         .get(&Statistic::UnboundInscriptions.key())?
    //         .map(|unbound_inscriptions| unbound_inscriptions.value())
    //         .unwrap_or(0);
    //
    //     let next_sequence_number = sequence_number_to_inscription_entry
    //         .iter()?
    //         .next_back()
    //         .and_then(|result| result.ok())
    //         .map(|(number, _id)| number.value() + 1)
    //         .unwrap_or(0);
    //
    //     let mut operations = HashMap::new();
    //     let mut processor = StorageProcessor::new(self.cache.clone(),self.internal_index.clone());
    //     // let mut processor = crate::index::updater::pending_updater::PendingStorageProcessor::new(
    //     //     &mut home_inscriptions,
    //     //     &mut inscription_id_to_sequence_number,
    //     //     &mut inscription_number_to_sequence_number,
    //     //     &mut outpoint_to_entry,
    //     //     &mut transaction_id_to_transaction,
    //     //     &mut sat_to_sequence_number,
    //     //     &mut satpoint_to_sequence_number,
    //     //     &mut sequence_number_to_children,
    //     //     &mut sequence_number_to_inscription_entry,
    //     //     &mut sequence_number_to_satpoint,
    //     // );
    //     let mut inscription_updater = PendingUpdater::new(
    //         &mut operations,
    //         blessed_inscription_count,
    //         self.internal_index.options.chain(),
    //         cursed_inscription_count,
    //         height,
    //         self.internal_index.index_transactions,
    //         next_sequence_number,
    //         lost_sats,
    //         block.header.time,
    //         unbound_inscriptions,
    //         tx_out_receiver,
    //         tx_out_cache,
    //         &mut processor,
    //     )?;
    //
    //     let index_sats = true;
    //     if index_sats {
    //         let mut sat_to_satpoint = wtx.open_table(SAT_TO_SATPOINT)?;
    //         let mut outpoint_to_sat_ranges = wtx.open_table(OUTPOINT_TO_SAT_RANGES)?;
    //
    //         let mut coinbase_inputs = VecDeque::new();
    //
    //         let h = Height(height);
    //         if h.subsidy() > 0 {
    //             let start = h.starting_sat();
    //             coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
    //         }
    //
    //         for (tx_offset, (tx, txid)) in block.txdata.iter().enumerate().skip(1) {
    //             log::trace!("Indexing transaction {tx_offset}…");
    //
    //             let mut input_sat_ranges = VecDeque::new();
    //
    //             // TODO: make sure this is correct
    //             // for input in &tx.input {
    //             // let key = input.previous_output.store();
    //             // let sat_ranges = match self.range_cache.remove(&key) {
    //             //     Some(sat_ranges) => {
    //             //         self.outputs_cached += 1;
    //             //         sat_ranges
    //             //     }
    //             //     None => outpoint_to_sat_ranges
    //             //         .remove(&key)?
    //             //         .ok_or_else(|| anyhow!("Could not find outpoint {} in index", input.previous_output))?
    //             //         .value()
    //             //         .to_vec(),
    //             // };
    //             //
    //             // for chunk in sat_ranges.chunks_exact(11) {
    //             //     input_sat_ranges.push_back(SatRange::load(chunk.try_into().unwrap()));
    //             // }
    //             // }
    //
    //             self.index_transaction_sats(
    //                 tx,
    //                 *txid,
    //                 &mut sat_to_satpoint,
    //                 &mut input_sat_ranges,
    //                 &mut sat_ranges_written,
    //                 &mut outputs_in_block,
    //                 &mut inscription_updater,
    //                 index_inscriptions,
    //             )?;
    //
    //             coinbase_inputs.extend(input_sat_ranges);
    //         }
    //
    //         if let Some((tx, txid)) = block.txdata.first() {
    //             self.index_transaction_sats(
    //                 tx,
    //                 *txid,
    //                 &mut sat_to_satpoint,
    //                 &mut coinbase_inputs,
    //                 &mut sat_ranges_written,
    //                 &mut outputs_in_block,
    //                 &mut inscription_updater,
    //                 index_inscriptions,
    //             )?;
    //         }
    //
    //         if !coinbase_inputs.is_empty() {
    //             let mut lost_sat_ranges = outpoint_to_sat_ranges
    //                 .remove(&OutPoint::null().store())?
    //                 .map(|ranges| ranges.value().to_vec())
    //                 .unwrap_or_default();
    //
    //             for (start, end) in coinbase_inputs {
    //                 if !Sat(start).common() {
    //                     sat_to_satpoint.insert(
    //                         &start,
    //                         &SatPoint {
    //                             outpoint: OutPoint::null(),
    //                             offset: lost_sats,
    //                         }
    //                             .store(),
    //                     )?;
    //                 }
    //
    //                 lost_sat_ranges.extend_from_slice(&(start, end).store());
    //
    //                 lost_sats += end - start;
    //             }
    //
    //             outpoint_to_sat_ranges.insert(&OutPoint::null().store(), lost_sat_ranges.as_slice())?;
    //         }
    //     } else if index_inscriptions {
    //         for (tx, txid) in block.txdata.iter().skip(1).chain(block.txdata.first()) {
    //             inscription_updater.index_envelopes(tx, *txid, None)?;
    //         }
    //     }
    //     inscription_updater.flush_cache()?;
    //
    //     let mut context = Context {
    //         chain: BlockContext {
    //             network: index.get_chain_network(),
    //             blockheight: height,
    //             blocktime: block.header.time,
    //         },
    //         tx_out_cache,
    //         hit: 0,
    //         miss: 0,
    //         ORD_TX_TO_OPERATIONS: &mut wtx.open_table(ORD_TX_TO_OPERATIONS)?,
    //         COLLECTIONS_KEY_TO_INSCRIPTION_ID: &mut wtx.open_table(COLLECTIONS_KEY_TO_INSCRIPTION_ID)?,
    //         COLLECTIONS_INSCRIPTION_ID_TO_KINDS: &mut wtx
    //             .open_table(COLLECTIONS_INSCRIPTION_ID_TO_KINDS)?,
    //         SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY: &mut sequence_number_to_inscription_entry,
    //         OUTPOINT_TO_ENTRY: &mut outpoint_to_entry,
    //         BRC20_BALANCES: &mut wtx.open_table(BRC20_BALANCES)?,
    //         BRC20_TOKEN: &mut wtx.open_table(BRC20_TOKEN)?,
    //         BRC20_EVENTS: &mut wtx.open_table(BRC20_EVENTS)?,
    //         BRC20_TRANSFERABLELOG: &mut wtx.open_table(BRC20_TRANSFERABLELOG)?,
    //         BRC20_INSCRIBE_TRANSFER: &mut wtx.open_table(BRC20_INSCRIBE_TRANSFER)?,
    //     };
    //
    //     // Create a protocol manager to index the block of bitmap data.
    //     let config = ProtocolConfig::new_with_options(&index.options);
    //     ProtocolManager::new(config).index_block(&mut context, &block, operations)?;
    //
    //     Ok(())
    // }

    fn index_block(
        &mut self,
        height: u32,
        outpoint_sender: &'a mut Sender<OutPoint>,
        tx_out_receiver: &'a mut Receiver<TxOut>,
        block: BlockData,
        tx_out_cache: &'a mut SimpleLru<OutPoint, TxOut>,
        processor: &'db mut StorageProcessor<'a, 'db, 'tx>,
        operations: &'db mut HashMap<Txid, Vec<InscriptionOp>>,
    ) -> crate::Result<()> {
        let start = Instant::now();
        let mut sat_ranges_written = 0;
        let mut outputs_in_block = 0;

        // If value_receiver still has values something went wrong with the last block
        // Could be an assert, shouldn't recover from this and commit the last block
        let Err(TryRecvError::Empty) = tx_out_receiver.try_recv() else {
            return Err(anyhow!("Previous block did not consume all input values"));
        };


        let index_inscriptions = true;

        let fetching_outputs_count = AtomicUsize::new(0);
        let total_outputs_count = AtomicUsize::new(0);
        let cache_outputs_count = AtomicUsize::new(0);
        let miss_outputs_count = AtomicUsize::new(0);
        let meet_outputs_count = AtomicUsize::new(0);
        if index_inscriptions {
            // Send all missing input outpoints to be fetched right away
            let txids = block
                .txdata
                .iter()
                .map(|(_, txid)| txid)
                .collect::<HashSet<_>>();
            use rayon::prelude::*;
            let tx_outs = block
                .txdata
                .par_iter()
                .flat_map(|(tx, _)| tx.input.par_iter())
                .filter_map(|input| {
                    total_outputs_count.fetch_add(1, Ordering::Relaxed);
                    let prev_output = input.previous_output;
                    // We don't need coinbase input value
                    if prev_output.is_null() {
                        None
                    } else if txids.contains(&prev_output.txid) {
                        meet_outputs_count.fetch_add(1, Ordering::Relaxed);
                        None
                    } else if tx_out_cache.contains(&prev_output) {
                        cache_outputs_count.fetch_add(1, Ordering::Relaxed);
                        None
                    } else if let Some(txout) = processor.get_txout_by_outpoint(&prev_output).unwrap()
                    {
                        miss_outputs_count.fetch_add(1, Ordering::Relaxed);
                        Some((prev_output, Some(txout)))
                    } else {
                        fetching_outputs_count.fetch_add(1, Ordering::Relaxed);
                        Some((prev_output, None))
                    }
                })
                .collect::<Vec<_>>();
            for (out_point, value) in tx_outs.into_iter() {
                if let Some(tx_out) = value {
                    tx_out_cache.insert(out_point, tx_out);
                } else {
                    outpoint_sender.blocking_send(out_point).unwrap();
                }
            }
        }

        let mut lost_sats = processor.get_lost_sats()?;
        let cursed_inscription_count = processor.get_cursed_inscription_count()?;
        let blessed_inscription_count = processor.get_blessed_inscription_count()?;
        let unbound_inscriptions = processor.get_unbound_inscriptions()?;
        let next_sequence_number = processor.next_sequence_number()?;

        // let processor = &mut self.processor;
        let mut inscription_updater = PendingUpdater::new(
            operations,
            blessed_inscription_count,
            self.internal_index.options.chain(),
            cursed_inscription_count,
            height,
            self.internal_index.index_transactions,
            next_sequence_number,
            lost_sats,
            block.header.time,
            unbound_inscriptions,
            tx_out_receiver,
            tx_out_cache,
            processor,
        )?;

        let index_sats = true;
        if index_sats {
            let mut coinbase_inputs = VecDeque::new();

            let h = Height(height);
            if h.subsidy() > 0 {
                let start = h.starting_sat();
                coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
            }

            for (tx_offset, (tx, txid)) in block.txdata.iter().enumerate().skip(1) {
                log::trace!("Indexing transaction {tx_offset}…");

                let mut input_sat_ranges = VecDeque::new();

                self.index_transaction_sats(
                    tx,
                    *txid,
                    &mut input_sat_ranges,
                    &mut sat_ranges_written,
                    &mut outputs_in_block,
                    &mut inscription_updater,
                    index_inscriptions,
                )?;

                coinbase_inputs.extend(input_sat_ranges);
            }

            if let Some((tx, txid)) = block.txdata.first() {
                self.index_transaction_sats(
                    tx,
                    *txid,
                    &mut coinbase_inputs,
                    &mut sat_ranges_written,
                    &mut outputs_in_block,
                    &mut inscription_updater,
                    index_inscriptions,
                )?;
            }

            if !coinbase_inputs.is_empty() {
                let mut lost_sat_ranges = processor.outpoint_to_sat_ranges_remove(&OutPoint::null().store())?.map(|ranges| ranges.to_vec())
                    .unwrap_or_default();

                for (start, end) in coinbase_inputs {
                    if !Sat(start).common() {
                        processor.sat_to_satpoint_insert(
                            &start,
                            &SatPoint {
                                outpoint: OutPoint::null(),
                                offset: lost_sats,
                            }
                                .store(),
                        )?;
                    }

                    lost_sat_ranges.extend_from_slice(&(start, end).store());

                    lost_sats += end - start;
                }
                processor.outpoint_to_sat_ranges_insert(&OutPoint::null().store(), lost_sat_ranges.as_slice())?;
            }
        } else if index_inscriptions {
            for (tx, txid) in block.txdata.iter().skip(1).chain(block.txdata.first()) {
                inscription_updater.index_envelopes(tx, *txid, None)?;
            }
        }
        inscription_updater.flush_cache()?;

        // // TODO:
        // let mut context = processor.create_context()?;
        // // // Create a protocol manager to index the block of bitmap data.
        // let config = ProtocolConfig::new_with_options(&index.options);
        // ProtocolManager::new(config).index_block(&mut context, &block, operations.clone())?;

        Ok(())
    }


    fn index_and_execute(&self) {}
    fn index_transaction_sats(
        &mut self,
        tx: &Transaction,
        txid: Txid,
        input_sat_ranges: &mut VecDeque<(u64, u64)>,
        sat_ranges_written: &mut u64,
        outputs_traversed: &mut u64,
        inscription_updater: &mut PendingUpdater,
        index_inscriptions: bool,
    ) -> crate::Result {
        if index_inscriptions {
            inscription_updater.index_envelopes(tx, txid, Some(input_sat_ranges))?;
        }

        for (vout, output) in tx.output.iter().enumerate() {
            let outpoint = OutPoint {
                vout: vout.try_into().unwrap(),
                txid,
            };
            let mut sats = Vec::new();

            let mut remaining = output.value;
            while remaining > 0 {
                let range = input_sat_ranges
                    .pop_front()
                    .ok_or_else(|| anyhow!("insufficient inputs for transaction outputs"))?;

                if !Sat(range.0).common() {
                    inscription_updater.processor.sat_to_satpoint_insert(
                        &range.0,
                        &SatPoint {
                            outpoint,
                            offset: 0,
                        }
                            .store(),
                    )?;
                }

                let count = range.1 - range.0;

                let assigned = if count > remaining {
                    let middle = range.0 + remaining;
                    input_sat_ranges.push_front((middle, range.1));
                    (range.0, middle)
                } else {
                    range
                };

                sats.extend_from_slice(&assigned.store());

                remaining -= assigned.1 - assigned.0;

                *sat_ranges_written += 1;
            }

            *outputs_traversed += 1;
        }

        Ok(())
    }
}

#[test]
pub fn test_simulate() {
    let internal = Arc::new(Index::open(&Default::default()).unwrap());
    let sim = Simulator {
        options: Default::default(),
        // simulate_index: IndexTracer {},
        internal_index: internal.clone(),
        _marker_a: Default::default(),
        _marker_b: Default::default(),
        _marker_tx: Default::default(),
    };
    let simulate_index = Index::open(&Options {
        index: Some(PathBuf::from("./simulate")),
        index_runes: false,
        index_sats: true,
        index_transactions: true,
        regtest: true,
        enable_index_brc20: true,
        ..Default::default()
    }).unwrap();
    let mut wtx = simulate_index.begin_write().unwrap();

    let mut home_inscriptions = wtx.open_table(HOME_INSCRIPTIONS).unwrap();
    let mut inscription_id_to_sequence_number =
        wtx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER).unwrap();
    let mut inscription_number_to_sequence_number =
        wtx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER).unwrap();
    let mut sat_to_sequence_number = wtx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER).unwrap();
    let mut satpoint_to_sequence_number = wtx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER).unwrap();
    let mut sequence_number_to_children = wtx.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN).unwrap();
    let mut sequence_number_to_inscription_entry =
        wtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY).unwrap();
    let mut sequence_number_to_satpoint = wtx.open_table(SEQUENCE_NUMBER_TO_SATPOINT).unwrap();
    let mut transaction_id_to_transaction = wtx.open_table(TRANSACTION_ID_TO_TRANSACTION).unwrap();
    let mut outpoint_to_entry = wtx.open_table(OUTPOINT_TO_ENTRY).unwrap();

    let processor = StorageProcessor {
        internal: internal.clone(),
        // wtx: &mut wtx,
        home_inscriptions: &mut home_inscriptions,
        id_to_sequence_number: &mut inscription_id_to_sequence_number,
        inscription_number_to_sequence_number: &mut inscription_number_to_sequence_number,
        outpoint_to_entry: &mut outpoint_to_entry,
        transaction_id_to_transaction: &mut transaction_id_to_transaction,
        sat_to_sequence_number: &mut sat_to_sequence_number,
        satpoint_to_sequence_number: &mut satpoint_to_sequence_number,
        sequence_number_to_children: &mut sequence_number_to_children,
        sequence_number_to_entry: &mut sequence_number_to_inscription_entry,
        sequence_number_to_satpoint: &mut sequence_number_to_satpoint,
    };
}