use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use anyhow::anyhow;
use bitcoin::{OutPoint, Transaction, Txid, TxOut};
use indexer_sdk::client::drect::DirectClient;
use indexer_sdk::client::event::ClientEvent;
use indexer_sdk::client::SyncClient;
use indexer_sdk::configuration::base::{IndexerConfiguration, NetConfiguration, ZMQConfiguration};
use indexer_sdk::factory::common::async_create_and_start_processor;
use indexer_sdk::storage::db::memory::MemoryDB;
use indexer_sdk::storage::db::thread_safe::ThreadSafeDB;
use indexer_sdk::storage::kv::KVStorageProcessor;
use indexer_sdk::wait_exit_signal;
use log::{error, info};
use redb::{WriteTransaction};
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use crate::{Index, Options, Sat, SatPoint};
use crate::height::Height;
use crate::index::{BlockData, BRC20_BALANCES, BRC20_EVENTS, BRC20_INSCRIBE_TRANSFER, BRC20_TOKEN, BRC20_TRANSFERABLELOG, COLLECTIONS_INSCRIPTION_ID_TO_KINDS, COLLECTIONS_KEY_TO_INSCRIPTION_ID, HOME_INSCRIPTIONS, INSCRIPTION_ID_TO_SEQUENCE_NUMBER, INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, OUTPOINT_TO_ENTRY, OUTPOINT_TO_SAT_RANGES, SAT_TO_SATPOINT, SAT_TO_SEQUENCE_NUMBER, SATPOINT_TO_SEQUENCE_NUMBER, SEQUENCE_NUMBER_TO_CHILDREN, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, SEQUENCE_NUMBER_TO_SATPOINT, STATISTIC_TO_COUNT, TRANSACTION_ID_TO_TRANSACTION};
use crate::index::entry::Entry;
use crate::index::simulator::error::SimulateError;
use crate::index::simulator::processor::{IndexWrapper, StorageProcessor};
use crate::index::updater::pending_updater::PendingUpdater;
use crate::okx::datastore::brc20::{Brc20Reader, Receipt};
use crate::okx::datastore::ord::InscriptionOp;
use crate::okx::lru::SimpleLru;
use crate::okx::protocol::{ProtocolConfig, ProtocolManager};
use crate::okx::protocol::simulate::SimulateContext;
use crate::okx::protocol::trace::TraceNode;

pub struct Simulator<'a, 'db, 'tx> {
    pub internal_index: IndexWrapper,
    pub client: Option<DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>>,
    _marker_a: PhantomData<&'a ()>,
    _marker_b: PhantomData<&'db ()>,
    _marker_tx: PhantomData<&'tx ()>,
}

#[derive(Clone)]
pub struct SimulatorServer {
    tx_out_cache: Rc<RefCell<SimpleLru<OutPoint, TxOut>>>,
    pub internal_index: IndexWrapper,
    pub simulate_index: Arc<Index>,
    pub client: DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>,
}

unsafe impl Send for SimulatorServer {}

unsafe impl Sync for SimulatorServer {}

impl SimulatorServer {
    pub async fn start(&self, exit: watch::Receiver<()>) -> JoinHandle<()> {
        let internal = self.clone();
        tokio::spawn(async move {
            internal.on_start(exit).await;
        })
    }
    async fn on_start(self, mut exit: watch::Receiver<()>) {
        let client = self.client.clone();
        loop {
            tokio::select! {
                     event=self.get_client_event()=>{
                         match event{
                            Ok(event) => {
                                if let Err(e)= self.handle_event(&event).await{
                                        log::error!("handle event error: {:?}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("receive event error: {:?}", e);
                                break;
                            }
                        }
                     },
                     _ = exit.changed() => {
                    info!("simulator receive exit signal, exit.");
                    break;
                 }
                    }
        }
    }
    async fn handle_event(&self, event: &ClientEvent) -> crate::Result<()> {
        info!("sim receive event:{:?}", event);
        match event {
            ClientEvent::Transaction(tx) => {
                self.execute_tx(tx, true)?;
            }
            ClientEvent::GetHeight => {}
            ClientEvent::TxDroped(_) => {}
            ClientEvent::TxConfirmed(_) => {}
        }
        Ok(())
    }
    async fn get_client_event(&self) -> crate::Result<ClientEvent, SimulateError> {
        let ret = self.client.block_get_event()?;
        Ok(ret)
    }
    pub fn execute_tx(&self, tx: &Transaction, commit: bool) -> crate::Result<Vec<Receipt>, SimulateError> {
        let mut wtx = self.simulate_index.begin_write()?;
        let traces = Rc::new(RefCell::new(vec![]));
        let ret = self.simulate_tx(tx, &wtx, traces)?;
        if commit {
            wtx.commit()?;
        }

        Ok(ret)
    }

    fn simulate_tx(&self, tx: &Transaction, wtx: &WriteTransaction, traces: Rc<RefCell<Vec<TraceNode>>>) -> crate::Result<Vec<Receipt>, SimulateError> {
        let brc20_receipts = Rc::new(RefCell::new(vec![]));
        let height = self.internal_index.internal.block_count()?;
        let block = self.internal_index.internal.get_block_by_height(height)?.unwrap();
        let home_inscriptions = wtx.open_table(HOME_INSCRIPTIONS).unwrap();
        let inscription_id_to_sequence_number =
            wtx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER).unwrap();
        let inscription_number_to_sequence_number =
            wtx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER).unwrap();
        let sat_to_sequence_number = wtx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER).unwrap();
        let satpoint_to_sequence_number = wtx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER).unwrap();
        let sequence_number_to_children = wtx.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN).unwrap();
        let mut sequence_number_to_inscription_entry =
            Rc::new(RefCell::new(wtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY).unwrap()));
        let sequence_number_to_satpoint = wtx.open_table(SEQUENCE_NUMBER_TO_SATPOINT).unwrap();
        let transaction_id_to_transaction = wtx.open_table(TRANSACTION_ID_TO_TRANSACTION).unwrap();
        let outpoint_to_entry = Rc::new(RefCell::new(wtx.open_table(OUTPOINT_TO_ENTRY).unwrap()));
        let OUTPOINT_TO_SAT_RANGES_table = wtx.open_table(OUTPOINT_TO_SAT_RANGES).unwrap();
        let sat_to_point = wtx.open_table(SAT_TO_SATPOINT).unwrap();
        let statis_to_count = wtx.open_table(STATISTIC_TO_COUNT).unwrap();

        let h = height;
        let ts = block.header.time;
        let ctx = SimulateContext {
            network: self.internal_index.internal.get_chain_network().clone(),
            current_height: h,
            current_block_time: ts as u32,
            internal_index: self.internal_index.clone(),
            ORD_TX_TO_OPERATIONS: Rc::new(RefCell::new(wtx.open_table(crate::index::ORD_TX_TO_OPERATIONS)?)),
            COLLECTIONS_KEY_TO_INSCRIPTION_ID: Rc::new(RefCell::new(wtx.open_table(COLLECTIONS_KEY_TO_INSCRIPTION_ID)?)),
            COLLECTIONS_INSCRIPTION_ID_TO_KINDS: Rc::new(RefCell::new(wtx
                .open_table(COLLECTIONS_INSCRIPTION_ID_TO_KINDS)?)),
            SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY: sequence_number_to_inscription_entry.clone(),
            OUTPOINT_TO_ENTRY: outpoint_to_entry.clone(),
            BRC20_BALANCES: Rc::new(RefCell::new(wtx.open_table(BRC20_BALANCES)?)),
            BRC20_TOKEN: Rc::new(RefCell::new(wtx.open_table(BRC20_TOKEN)?)),
            BRC20_EVENTS: Rc::new(RefCell::new(wtx.open_table(BRC20_EVENTS)?)),
            BRC20_TRANSFERABLELOG: Rc::new(RefCell::new(wtx.open_table(BRC20_TRANSFERABLELOG)?)),
            BRC20_INSCRIBE_TRANSFER: Rc::new(RefCell::new(wtx.open_table(BRC20_INSCRIBE_TRANSFER)?)),
            traces: traces.clone(),
            brc20_receipts: brc20_receipts.clone(),
            _marker_a: Default::default(),
        };

        let db_receipts = ctx.get_transaction_receipts(&tx.txid())?;
        if db_receipts.len() > 0 {
            info!("tx:{:?} already simulated",tx.txid());
            return Ok(db_receipts);
        }

        let processor = StorageProcessor {
            internal: self.internal_index.clone(),
            // wtx: &mut wtx,
            home_inscriptions: Rc::new(RefCell::new(home_inscriptions)),
            id_to_sequence_number: Rc::new(RefCell::new(inscription_id_to_sequence_number)),
            inscription_number_to_sequence_number: Rc::new(RefCell::new(inscription_number_to_sequence_number)),
            outpoint_to_entry,
            transaction_id_to_transaction: Rc::new(RefCell::new(transaction_id_to_transaction)),
            sat_to_sequence_number: Rc::new(RefCell::new(sat_to_sequence_number)),
            satpoint_to_sequence_number: Rc::new(RefCell::new(satpoint_to_sequence_number)),
            sequence_number_to_children: Rc::new(RefCell::new(sequence_number_to_children)),
            sequence_number_to_satpoint: Rc::new(RefCell::new(sequence_number_to_satpoint)),
            sequence_number_to_inscription_entry,
            outpoint_to_sat_ranges: Rc::new(RefCell::new(OUTPOINT_TO_SAT_RANGES_table)),
            sat_to_satpoint: Rc::new(RefCell::new(sat_to_point)),
            statistic_to_count: Rc::new(RefCell::new(statis_to_count)),
            _marker_a: Default::default(),
            client: Some(self.client.clone()),
            traces: traces.clone(),
            context: ctx,
        };

        self.loop_simulate_tx(&processor, &tx)?;
        let ret = brc20_receipts.borrow();
        let ret = ret.deref().clone();
        Ok(ret)
    }
    pub fn loop_simulate_tx(&self, processor: &StorageProcessor, tx: &Transaction) -> crate::Result<(), SimulateError> {
        let tx_id = tx.txid();
        let mut need_handle_first = vec![];
        for input in &tx.input {
            if input.previous_output.is_null() {
                continue;
            }
            let prev_tx_id = &input.previous_output.txid;
            let prev_out = processor.get_txout_by_outpoint(&input.previous_output)?;
            if prev_out.is_none() {
                need_handle_first.push(prev_tx_id);
            }
        }
        if need_handle_first.is_empty() {
            info!("parent suits is ready,start to simulate tx:{:?}",&tx_id);
        }
        for parent in need_handle_first {
            let parent_tx = processor.get_transaction(&parent)?;
            if parent_tx.is_none() {
                error!("parent tx not exist,tx_hash:{:?},child_hash:{:?}",&parent,&tx_id);
                return Err(SimulateError::TxNotFound(parent.clone()));
            }
            let parent_tx = parent_tx.unwrap();
            info!("parent tx :{:?},exist,but not in utxo data,child_hash:{:?},need to simulate parent tx",&parent,&tx_id);
            self.loop_simulate_tx(processor, &parent_tx)?;
            info!("parent tx {:?} simulate done,start to simulate child_hash:{:?}",&parent,&tx_id);
        }
        self.do_simulate_tx(processor, &tx)?;

        Ok(())
    }

    pub fn do_simulate_tx(&self, processor: &StorageProcessor, tx: &Transaction) -> crate::Result<(), SimulateError> {
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

        let mut operations: HashMap<Txid, Vec<InscriptionOp>> = HashMap::new();
        let block = BlockData {
            header: block.header,
            txdata: vec![(tx.clone(), tx.txid())],
        };
        sim.index_block(block.clone(), height, cache, &processor, &mut operations)?;

        Ok(())
    }

    pub fn new(ops: Options, internal_index: Arc<Index>, simulate_ops: Option<Options>, client: DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>) -> crate::Result<Self> {
        let simulate_index = if let Some(ops) = simulate_ops {
            Index::open(&ops)?
        } else {
            let mut origin_ops = ops.clone();
            let dbfile = NamedTempFile::new().unwrap();
            origin_ops.index = Some(dbfile.path().to_path_buf());
            Index::open(&origin_ops)?
        };
        let simulate_index = Arc::new(simulate_index);
        Ok(Self { tx_out_cache: Rc::new(RefCell::new(SimpleLru::new(500))), internal_index: IndexWrapper::new(internal_index), simulate_index, client })
    }
}

impl<'a, 'db, 'tx> Simulator<'a, 'db, 'tx> {
    fn index_block(
        &mut self,
        block: BlockData,
        height: u32,
        tx_out_cache: &'a mut SimpleLru<OutPoint, TxOut>,
        processor: &StorageProcessor<'a, 'db, 'tx>,
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
                    let tx = processor.get_transaction(&out_point.txid)?.unwrap();
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

        let mut context = processor.create_context()?;
        let config = ProtocolConfig::new_with_options(&self.internal_index.internal.options);
        ProtocolManager::new(config).index_block(&mut context, &block, operations.clone())?;

        Ok(())
    }

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

pub fn start_simulator(ops: Options, internal: Arc<Index>) -> Option<SimulatorServer> {
    if !ops.simulate_enable {
        return None;
    }
    let rt = Runtime::new().unwrap();
    let zmq_url = ops.simulate_zmq_url.clone().unwrap();
    let sim_rpc = ops.simulate_bitcoin_rpc_url.clone().unwrap();
    let sim_user = ops.simulate_bitcoin_rpc_user.clone().unwrap();
    let sim_pass = ops.simulate_bitcoin_rpc_pass.clone().unwrap();

    let config = IndexerConfiguration {
        mq: ZMQConfiguration { zmq_url: zmq_url, zmq_topic: vec!["sequence".to_string(), "rawblock".to_string()] },
        net: NetConfiguration {
            url: sim_rpc,
            username: sim_user,
            password: sim_pass,
        },
        ..Default::default()
    };
    let (tx, rx) = watch::channel(());
    let (server, handlers) = rt.block_on(async {
        let ret = async_create_and_start_processor(rx.clone(), config).await;
        let mut handlers = ret.1;
        let mut sim_ops = ops.clone();
        sim_ops.index = ops.simulate_index.clone();
        let server = SimulatorServer::new(ops.clone(), internal.clone(), Some(sim_ops), ret.0).unwrap();
        handlers.push(server.start(rx.clone()).await);
        (server.clone(), handlers)
    });

    thread::spawn(move || {
        rt.block_on(async {
            wait_exit_signal().await.unwrap();
            tx.send(()).unwrap();
            for h in handlers {
                h.await.unwrap();
            }
        });
    });
    Some(server)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::thread::sleep;
    use bitcoincore_rpc::RpcApi;
    use indexer_sdk::factory::common::new_client_for_test;
    use log::LevelFilter;
    use super::*;

    #[test]
    pub fn test_start() {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .format_target(false)
            .init();
        let opt = create_options();
        let internal = Arc::new(Index::open(&opt).unwrap());
        let server = start_simulator(opt, internal.clone());
        sleep(std::time::Duration::from_secs(5))
    }

    #[test]
    pub fn test_simulate_tx() {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .format_target(false)
            .init();
        let opt = create_options();
        let internal = Arc::new(Index::open(&opt).unwrap());
        let mut opt2 = opt.clone();
        opt2.index=opt.simulate_index.clone();
        let client = new_client_for_test("http://localhost:18443".to_string(), "bitcoinrpc".to_string(), "bitcoinrpc".to_string());
        let simulate_server = SimulatorServer::new(internal.options.clone(), internal.clone(), Some(opt2), client.clone()).unwrap();

        let client = client.clone().get_btc_client();
        let tx = client.get_raw_transaction(&Txid::from_str("f9028dbd87d723399181d9bdb80a36e991b56405dfae2ccb6ee033d249b5f724").unwrap(), None).unwrap();
        println!("{:?}", tx);
        simulate_server.execute_tx(&tx, true).unwrap();
    }

    fn create_options() -> Options {
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
            index: Some(PathBuf::from("./internal")),
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
            simulate_enable: true,
            simulate_zmq_url: Some("tcp://0.0.0.0:28332".to_string()),
            simulate_bitcoin_rpc_url: Some("http://localhost:18443".to_string()),
            simulate_bitcoin_rpc_pass: Some("bitcoinrpc".to_string()),
            simulate_bitcoin_rpc_user: Some("bitcoinrpc".to_string()),
            simulate_index: Some("./simulate".to_string().into()),
        };
        opt
    }
}
