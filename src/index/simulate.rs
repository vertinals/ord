use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::anyhow;
use bitcoin::{OutPoint, Transaction, Txid, TxOut};
use indexer_sdk::client::drect::DirectClient;
use indexer_sdk::client::SyncClient;
use indexer_sdk::storage::db::memory::MemoryDB;
use indexer_sdk::storage::db::thread_safe::ThreadSafeDB;
use indexer_sdk::storage::kv::KVStorageProcessor;
use redb::{ReadableTable};
use crate::{Index, Options, Rune, RuneEntry, Sat, SatPoint, timestamp};
use crate::height::Height;
use crate::index::{BlockData, BRC20_BALANCES, BRC20_EVENTS, BRC20_INSCRIBE_TRANSFER, BRC20_TOKEN, BRC20_TRANSFERABLELOG, COLLECTIONS_INSCRIPTION_ID_TO_KINDS, COLLECTIONS_KEY_TO_INSCRIPTION_ID, HEIGHT_TO_BLOCK_HEADER, HEIGHT_TO_LAST_SEQUENCE_NUMBER, HOME_INSCRIPTIONS, INSCRIPTION_ID_TO_SEQUENCE_NUMBER, INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, ORD_TX_TO_OPERATIONS, OUTPOINT_TO_ENTRY, OUTPOINT_TO_RUNE_BALANCES, OUTPOINT_TO_SAT_RANGES, RUNE_ID_TO_RUNE_ENTRY, RUNE_TO_RUNE_ID, SAT_TO_SATPOINT, SAT_TO_SEQUENCE_NUMBER, SATPOINT_TO_SEQUENCE_NUMBER, SEQUENCE_NUMBER_TO_CHILDREN, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, SEQUENCE_NUMBER_TO_RUNE_ID, SEQUENCE_NUMBER_TO_SATPOINT, Statistic, STATISTIC_TO_COUNT, TRANSACTION_ID_TO_RUNE, TRANSACTION_ID_TO_TRANSACTION};
use crate::index::entry::{Entry, SatPointValue, SatRange};
use crate::index::processor::{IndexWrapper, StorageProcessor};
use crate::index::updater::pending_updater::PendingUpdater;
use crate::okx::datastore::cache::CacheWriter;
use crate::okx::datastore::ord::InscriptionOp;
use crate::okx::datastore::ord::redb::table::get_txout_by_outpoint;
use crate::okx::lru::SimpleLru;
use crate::okx::protocol::{BlockContext, ProtocolConfig, ProtocolManager};
use crate::okx::protocol::context::Context;
use crate::okx::protocol::trace::IndexTracer;

pub struct Simulator<'a, 'db, 'tx> {
    // pub simulate_index: IndexTracer,
    pub internal_index: IndexWrapper,
    pub client: Option<DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>>,
    _marker_a: PhantomData<&'a ()>,
    _marker_b: PhantomData<&'db ()>,
    _marker_tx: PhantomData<&'tx ()>,
}

pub struct SimulatorServer {
    tx_out_cache: Rc<RefCell<SimpleLru<OutPoint, TxOut>>>,
    pub internal_index: IndexWrapper,
    pub simulate_index: Arc<Index>,
    pub client: DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>,
}

impl SimulatorServer {
    pub fn simulate_tx(&self, tx: &Transaction) -> crate::Result<()> {
        let mut sim = Simulator {
            internal_index: self.internal_index.clone(),
            client: None,
            _marker_a: Default::default(),
            _marker_b: Default::default(),
            _marker_tx: Default::default(),
        };
        let height = self.internal_index.internal.block_count()?;
        let block = self.internal_index.internal.get_block_by_height(height)?.unwrap();
        let mut cache = self.tx_out_cache.borrow_mut();
        let cache = cache.deref_mut();

        let mut wtx = self.simulate_index.begin_write()?;
        // let wtx = Rc::new(RefCell::new(wtx));
        let binding = wtx;
        let home_inscriptions = binding.open_table(HOME_INSCRIPTIONS).unwrap();
        let inscription_id_to_sequence_number =
            binding.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER).unwrap();
        let inscription_number_to_sequence_number =
            binding.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER).unwrap();
        let sat_to_sequence_number = binding.open_multimap_table(SAT_TO_SEQUENCE_NUMBER).unwrap();
        let satpoint_to_sequence_number = binding.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER).unwrap();
        let sequence_number_to_children = binding.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN).unwrap();
        let mut sequence_number_to_inscription_entry =
            binding.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY).unwrap();
        let sequence_number_to_satpoint = binding.open_table(SEQUENCE_NUMBER_TO_SATPOINT).unwrap();
        let transaction_id_to_transaction = binding.open_table(TRANSACTION_ID_TO_TRANSACTION).unwrap();
        let outpoint_to_entry = binding.open_table(OUTPOINT_TO_ENTRY).unwrap();
        let OUTPOINT_TO_SAT_RANGES_table = binding.open_table(OUTPOINT_TO_SAT_RANGES).unwrap();
        let sat_to_point = binding.open_table(SAT_TO_SATPOINT).unwrap();
        let statis_to_count = binding.open_table(STATISTIC_TO_COUNT).unwrap();

        let processor = StorageProcessor {
            internal: self.internal_index.clone(),
            // wtx: &mut wtx,
            home_inscriptions: Rc::new(RefCell::new(home_inscriptions)),
            id_to_sequence_number: Rc::new(RefCell::new(inscription_id_to_sequence_number)),
            inscription_number_to_sequence_number: Rc::new(RefCell::new(inscription_number_to_sequence_number)),
            outpoint_to_entry: Rc::new(RefCell::new(outpoint_to_entry)),
            transaction_id_to_transaction: Rc::new(RefCell::new(transaction_id_to_transaction)),
            sat_to_sequence_number: Rc::new(RefCell::new(sat_to_sequence_number)),
            satpoint_to_sequence_number: Rc::new(RefCell::new(satpoint_to_sequence_number)),
            sequence_number_to_children: Rc::new(RefCell::new(sequence_number_to_children)),
            sequence_number_to_satpoint: Rc::new(RefCell::new(sequence_number_to_satpoint)),
            sequence_number_to_inscription_entry: Rc::new(RefCell::new((sequence_number_to_inscription_entry))),
            OUTPOINT_TO_SAT_RANGES: Rc::new(RefCell::new(OUTPOINT_TO_SAT_RANGES_table)),
            sat_to_satpoint: Rc::new(RefCell::new((sat_to_point))),
            statistic_to_count: Rc::new(RefCell::new((statis_to_count))),
            _marker_a: Default::default(),
            client: Some(self.client.clone()),
        };
        let mut operations: HashMap<Txid, Vec<InscriptionOp>> = HashMap::new();
        sim.index_block(BlockData {
            header: block.header,
            txdata: vec![(tx.clone(), tx.txid())],
        }, height, cache, processor, &mut operations)?;

        binding.commit()?;

        Ok(())
    }
}

impl<'a, 'db, 'tx> Simulator<'a, 'db, 'tx> {
    fn index_block(
        &mut self,
        block: BlockData,
        height: u32,
        tx_out_cache: &'a mut SimpleLru<OutPoint, TxOut>,
        processor: StorageProcessor<'a, 'db, 'tx>,
        operations: &'a mut HashMap<Txid, Vec<InscriptionOp>>,
    ) -> crate::Result<()> {
        let mut sat_ranges_written = 0;
        let mut outputs_in_block = 0;


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
                    let tx = processor.get_transaction(&out_point.txid)?;
                    let out = tx.output[out_point.vout as usize].clone();
                    let tx_out = TxOut {
                        value: out.value,
                        script_pubkey: out.script_pubkey.clone(),
                    };
                    tx_out_cache.insert(out_point, tx_out);
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
            self.internal_index.internal.options.chain(),
            cursed_inscription_count,
            height,
            self.internal_index.internal.index_transactions,
            next_sequence_number,
            lost_sats,
            block.header.time,
            unbound_inscriptions,
            tx_out_cache,
            processor.clone(),
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
        let mut context = processor.create_context()?;
        // // // Create a protocol manager to index the block of bitmap data.
        let config = ProtocolConfig::new_with_options(&self.internal_index.internal.options);
        ProtocolManager::new(config).index_block(&mut context, &block, operations.clone())?;

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
    let opt = crate::options::Options {
        log_level: Default::default(),
        log_dir: None,
        bitcoin_data_dir: None,
        bitcoin_rpc_pass: Some("bitcoinrpc".to_string()),
        bitcoin_rpc_user: Some("bitcoinrpc".to_string()),
        chain_argument: Default::default(),
        config: None,
        config_dir: None,
        cookie_file: None,
        data_dir: Default::default(),
        db_cache_size: None,
        lru_size: 0,
        first_inscription_height: None,
        height_limit: None,
        index: None,
        index_runes: false,
        index_sats: true,
        index_transactions: true,
        no_index_inscriptions: false,
        regtest: true,
        rpc_url: None,
        signet: false,
        testnet: false,
        enable_save_ord_receipts: true,
        enable_index_bitmap: true,
        enable_index_brc20: true,
        first_brc20_height: Some(0),
    };
    let internal = IndexWrapper::new(Arc::new(Index::open(&opt).unwrap()));
    let mut sim = Simulator {
        // simulate_index: IndexTracer {},
        internal_index: internal.clone(),
        client: None,
        _marker_a: Default::default(),
        _marker_b: Default::default(),
        _marker_tx: Default::default(),
    };
    let mut opt2 = opt.clone();
    opt2.index = Some(PathBuf::from("./simulate"));
    let simulate_index = Index::open(&opt2).unwrap();
    let mut wtx = simulate_index.begin_write().unwrap();
    let wtx = Rc::new(RefCell::new(wtx));
    let binding = wtx.borrow();
    let mut home_inscriptions = binding.open_table(HOME_INSCRIPTIONS).unwrap();
    let mut inscription_id_to_sequence_number =
        binding.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER).unwrap();
    let mut inscription_number_to_sequence_number =
        binding.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER).unwrap();
    let mut sat_to_sequence_number = binding.open_multimap_table(SAT_TO_SEQUENCE_NUMBER).unwrap();
    let mut satpoint_to_sequence_number = binding.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER).unwrap();
    let mut sequence_number_to_children = binding.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN).unwrap();
    let mut sequence_number_to_inscription_entry =
        binding.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY).unwrap();
    let mut sequence_number_to_satpoint = binding.open_table(SEQUENCE_NUMBER_TO_SATPOINT).unwrap();
    let mut transaction_id_to_transaction = binding.open_table(TRANSACTION_ID_TO_TRANSACTION).unwrap();
    let mut outpoint_to_entry = binding.open_table(OUTPOINT_TO_ENTRY).unwrap();
    let mut OUTPOINT_TO_SAT_RANGES_table = binding.open_table(OUTPOINT_TO_SAT_RANGES).unwrap();
    let sat_to_point = binding.open_table(SAT_TO_SATPOINT).unwrap();
    let statis_to_count = binding.open_table(STATISTIC_TO_COUNT).unwrap();
    let processor = StorageProcessor {
        internal: internal.clone(),
        // wtx: &mut wtx,
        home_inscriptions: Rc::new(RefCell::new(home_inscriptions)),
        id_to_sequence_number: Rc::new(RefCell::new(inscription_id_to_sequence_number)),
        inscription_number_to_sequence_number: Rc::new(RefCell::new(inscription_number_to_sequence_number)),
        outpoint_to_entry: Rc::new(RefCell::new(outpoint_to_entry)),
        transaction_id_to_transaction: Rc::new(RefCell::new(transaction_id_to_transaction)),
        sat_to_sequence_number: Rc::new(RefCell::new(sat_to_sequence_number)),
        satpoint_to_sequence_number: Rc::new(RefCell::new(satpoint_to_sequence_number)),
        sequence_number_to_children: Rc::new(RefCell::new(sequence_number_to_children)),
        sequence_number_to_satpoint: Rc::new(RefCell::new(sequence_number_to_satpoint)),
        sequence_number_to_inscription_entry: Rc::new(RefCell::new((sequence_number_to_inscription_entry))),
        OUTPOINT_TO_SAT_RANGES: Rc::new(RefCell::new(OUTPOINT_TO_SAT_RANGES_table)),
        sat_to_satpoint: Rc::new(RefCell::new((sat_to_point))),
        statistic_to_count: Rc::new(RefCell::new((statis_to_count))),
        _marker_a: Default::default(),
        client: None,
    };
}