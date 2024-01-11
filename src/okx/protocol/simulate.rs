use std::collections::HashMap;
use std::sync::Arc;
use bitcoin::{OutPoint, Txid, TxOut};
use redb::{Database, ReadableTable, ReadOnlyTable, ReadTransaction, RedbKey, RedbValue, Table, TableDefinition, WriteTransaction};
use tempfile::NamedTempFile;
use crate::{Index, InscriptionId};
use crate::index::{BRC20_BALANCES, BRC20_EVENTS, BRC20_INSCRIBE_TRANSFER, BRC20_TOKEN, BRC20_TRANSFERABLELOG, COLLECTIONS_INSCRIPTION_ID_TO_KINDS, COLLECTIONS_KEY_TO_INSCRIPTION_ID, InscriptionIdValue, ORD_TX_TO_OPERATIONS, OUTPOINT_TO_ENTRY, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY};
use crate::index::entry::Entry;
use crate::okx::datastore::brc20::{Balance, Brc20Reader, Brc20ReaderWriter, Receipt, Tick, TokenInfo, TransferableLog, TransferInfo};
use crate::okx::datastore::brc20::redb::{script_tick_id_key, script_tick_key};
use crate::okx::datastore::brc20::redb::table::{get_balance, get_balances, get_inscribe_transfer_inscription, get_token_info, get_tokens_info, get_transaction_receipts, get_transferable, get_transferable_by_id, get_transferable_by_tick};
use crate::okx::datastore::cache::CacheTableIndex;
use crate::okx::datastore::ScriptKey;
use crate::okx::lru::SimpleLru;
use crate::okx::protocol::BlockContext;
use crate::okx::protocol::context::Context;
use crate::okx::protocol::trace::{BalanceDelta, IndexTracer, MintTokenInfoDelta, string_to_bytes, TraceNode};

pub struct SimulateContext<'a, 'db, 'txn> {
    simulate: Context<'a, 'db, 'txn>,
    internal_index: Arc<Index>,
    // simulate_index: IndexTracer,
}

impl<'a, 'db, 'txn> SimulateContext<'a, 'db, 'txn> {
    pub fn new(
        internal_index: Arc<Index>, simulate: Context<'a, 'db, 'txn>) -> crate::Result<Self> {
        // let mut simulate_tx = simulate_index.begin_write()?;
        // let ctx = Context {
        //     chain,
        //     tx_out_cache,
        //     hit: 0,
        //     miss: 0,
        //     ORD_TX_TO_OPERATIONS: &mut simulate_tx.open_table(ORD_TX_TO_OPERATIONS)?,
        //     COLLECTIONS_KEY_TO_INSCRIPTION_ID: &mut simulate_tx.open_table(COLLECTIONS_KEY_TO_INSCRIPTION_ID)?,
        //     COLLECTIONS_INSCRIPTION_ID_TO_KINDS: &mut simulate_tx.open_table(COLLECTIONS_INSCRIPTION_ID_TO_KINDS)?,
        //     SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY: &mut simulate_tx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?,
        //     OUTPOINT_TO_ENTRY: &mut simulate_tx.open_table(OUTPOINT_TO_ENTRY)?,
        //     BRC20_BALANCES: &mut simulate_tx.open_table(BRC20_BALANCES)?,
        //     BRC20_TOKEN: &mut simulate_tx.open_table(BRC20_TOKEN)?,
        //     BRC20_EVENTS: &mut simulate_tx.open_table(BRC20_EVENTS)?,
        //     BRC20_TRANSFERABLELOG: &mut simulate_tx.open_table(BRC20_TRANSFERABLELOG)?,
        //     BRC20_INSCRIBE_TRANSFER: &mut simulate_tx.open_table(BRC20_INSCRIBE_TRANSFER)?,
        // };
        Ok(Self {
            // simulate: ctx,
            internal_index,
            simulate,
        })
    }
}

impl<'a, 'db, 'txn> SimulateContext<'a, 'db, 'txn> {
    fn use_internal_table<K: RedbKey + 'static, V: RedbValue + 'static, T>(&self,
                                                                           table_def: TableDefinition<K, V>,
                                                                           f: impl FnOnce(ReadOnlyTable<K, V>) -> crate::Result<T>) -> crate::Result<T> {
        let rtx = self.internal_index.begin_read()?;
        let table = rtx.0.open_table(table_def)?;
        let ret = f(table);
        ret
    }
}

// impl Brc20Reader for SimulateContext {
//     type Error = anyhow::Error;
//
//     fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
//         let simulate = self.simulate_index.use_read_table(BRC20_BALANCES, |table| {
//             get_balances(&table, script_key)
//         })?;
//         let mut simulate_balances: HashMap<Tick, Balance> = simulate.into_iter()
//             .map(|v| {
//                 (v.tick.clone(), v.clone())
//             }).collect();
//         let internal = self.use_internal_table(BRC20_BALANCES, |v| {
//             get_balances(&v, script_key)
//         })?;
//         for node in internal {
//             let v = simulate_balances.entry(node.tick.clone()).or_insert(node.clone());
//             v.transferable_balance = v.transferable_balance + node.transferable_balance;
//             v.overall_balance = v.overall_balance + node.overall_balance;
//         }
//         let ret = simulate_balances
//             .into_iter()
//             .map(|(k, v)| {
//                 v.clone()
//             }).collect();
//         Ok(ret)
//     }
//
//     fn get_balance(&self, script_key: &ScriptKey, tick: &Tick) -> crate::Result<Option<Balance>, Self::Error> {
//         let ret = self.simulate_index.use_read_table(BRC20_BALANCES, |table| {
//             get_balance(&table, script_key, tick)
//         })?;
//         if let Some(ret) = ret {
//             return Ok(Some(ret));
//         }
//         self.use_internal_table(BRC20_BALANCES, |table| {
//             get_balance(&table, script_key, tick)
//         })
//     }
//
//     fn get_token_info(&self, tick: &Tick) -> crate::Result<Option<TokenInfo>, Self::Error> {
//         let ret = self.simulate_index.use_read_table(BRC20_TOKEN, |table| {
//             get_token_info(&table, tick)
//         })?;
//         if let Some(ret) = ret {
//             return Ok(Some(ret));
//         }
//         self.use_internal_table(BRC20_TOKEN, |table| {
//             get_token_info(&table, tick)
//         })
//     }
//
//     fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
//         let simulate = self.simulate_index.use_read_table(BRC20_TOKEN, |table| {
//             get_tokens_info(&table)
//         })?;
//         let internal = self.use_internal_table(BRC20_TOKEN, |table| {
//             get_tokens_info(&table)
//         })?;
//         //     TODO:merge
//         todo!()
//     }
//
//     fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
//         let simulate = self.simulate_index.use_read_table(BRC20_EVENTS, |table| {
//             get_transaction_receipts(&table, txid)
//         })?;
//         let internal = self.use_internal_table(BRC20_EVENTS, |table| {
//             get_transaction_receipts(&table, txid)
//         })?;
//         //     TODO:merge
//         todo!()
//     }
//
//     fn get_transferable(&self, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>, Self::Error> {
//         let simulate = self.simulate_index.use_read_table(BRC20_TRANSFERABLELOG, |table| {
//             get_transferable(&table, script)
//         })?;
//         let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
//             get_transferable(&table, script)
//         })?;
//         // TODO: merge
//         todo!()
//     }
//
//
//     fn get_transferable_by_tick(&self, script: &ScriptKey, tick: &Tick) -> crate::Result<Vec<TransferableLog>, Self::Error> {
//         let simulate = self.simulate_index.use_read_table(BRC20_TRANSFERABLELOG, |table| {
//             get_transferable_by_tick(&table, script, tick)
//         })?;
//         let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
//             get_transferable_by_tick(&table, script, tick)
//         })?;
//         // TODO:merge
//         todo!()
//     }
//
//     fn get_transferable_by_id(&self, script: &ScriptKey, inscription_id: &InscriptionId) -> crate::Result<Option<TransferableLog>, Self::Error> {
//         let simulate = self.simulate_index.use_read_table(BRC20_TRANSFERABLELOG, |table| {
//             get_transferable_by_id(&table, script, inscription_id)
//         })?;
//         if let Some(ret) = simulate {
//             return Ok(Some(ret));
//         }
//         self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
//             get_transferable_by_id(&table, script, inscription_id)
//         })
//     }
//
//     fn get_inscribe_transfer_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<TransferInfo>, Self::Error> {
//         let simulate = self.simulate_index.use_read_table(BRC20_INSCRIBE_TRANSFER, |table| {
//             get_inscribe_transfer_inscription(&table, inscription_id)
//         })?;
//         if let Some(ret) = simulate {
//             return Ok(Some(ret));
//         }
//         self.use_internal_table(BRC20_INSCRIBE_TRANSFER, |table| {
//             get_inscribe_transfer_inscription(&table, inscription_id)
//         })
//     }
// }
//
// impl Brc20ReaderWriter for SimulateContext {
//     fn update_token_balance(&mut self, script_key: &ScriptKey, new_balance: Balance) -> crate::Result<(), Self::Error> {
//         self.simulate_index
//             .use_write_table(BRC20_BALANCES, |mut table| {
//                 let binding = script_tick_key(script_key, &new_balance.tick);
//                 let key = binding.as_str();
//                 let binding = rmp_serde::to_vec(&new_balance).unwrap();
//                 let value = binding.as_slice();
//                 // let origin = table.get(key)?.map_or(None, |v| {
//                 //     Some(v.value().to_vec())
//                 // });
//                 table.insert(key, value)?;
//                 // let mut delta = BalanceDelta::default();
//                 // if let Some(origin) = origin {
//                 //     let origin: Balance = rmp_serde::from_slice(&origin).unwrap();
//                 //     delta.origin_transferable_balance_delta = origin.transferable_balance;
//                 //     delta.origin_overall_balance_delta = origin.overall_balance;
//                 //     delta.new_overall_balance_delta = new_balance.overall_balance;
//                 //     delta.new_transferable_balance_delta = new_balance.transferable_balance;
//                 // } else {
//                 //     delta.origin_transferable_balance_delta = 0;
//                 //     delta.origin_overall_balance_delta = 0;
//                 //     delta.new_overall_balance_delta = new_balance.overall_balance;
//                 //     delta.new_transferable_balance_delta = new_balance.transferable_balance;
//                 // }
//                 // let op = TraceOperation::Update(rmp_serde::to_vec(&delta).unwrap());
//                 let node = TraceNode { trace_type: CacheTableIndex::BRC20_BALANCES, key: string_to_bytes(key) };
//                 Ok(((), node))
//             })?;
//         Ok(())
//     }
//
//     fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> crate::Result<(), Self::Error> {
//         self.simulate_index
//             .use_write_table(BRC20_TOKEN, |mut table| {
//                 let binding = tick.to_lowercase().hex();
//                 let key = binding.as_str();
//                 let binding = rmp_serde::to_vec(new_info).unwrap();
//                 let value = binding.as_slice();
//                 table.insert(key, value)?;
//
//                 let trace = TraceNode {
//                     trace_type: CacheTableIndex::BRC20_TOKEN,
//                     // operation: TraceOperation::Insert,
//                     key: string_to_bytes(key),
//                 };
//                 Ok(((), trace))
//             })?;
//         Ok(())
//     }
//
//
//     fn update_mint_token_info(&mut self, tick: &Tick, minted_amt: u128, minted_block_number: u32) -> crate::Result<(), Self::Error> {
//         let mut info = self.get_token_info(tick)?.unwrap_or_else(|| panic!("token {} not exist", tick.as_str()));
//         let origin = info.minted;
//         info.minted = minted_amt;
//         info.latest_mint_number = minted_block_number;
//         self.simulate_index
//             .use_write_table(BRC20_TOKEN, |mut table| {
//                 let binding = tick.to_lowercase().hex();
//                 let key = binding.as_str();
//                 let binding = rmp_serde::to_vec(&info).unwrap();
//                 let value = binding.as_slice();
//                 table.insert(key, value)?;
//                 // let delta = MintTokenInfoDelta {
//                 //     origin_minted: origin,
//                 //     new_minted: info.minted,
//                 //     new_latest_mint_number: minted_block_number,
//                 // };
//                 let trace = TraceNode {
//                     trace_type: CacheTableIndex::BRC20_TOKEN,
//                     // operation: TraceOperation::Update(rmp_serde::to_vec(&delta).unwrap()),
//                     key: string_to_bytes(key),
//                 };
//                 Ok(((), trace))
//             })?;
//         Ok(())
//     }
//
//     fn save_transaction_receipts(&mut self, txid: &Txid, receipt: &[Receipt]) -> crate::Result<(), Self::Error> {
//         self.simulate_index
//             .use_write_table(BRC20_EVENTS, |mut table| {
//                 let tx_id_value = txid.store();
//                 let key = tx_id_value.to_vec();
//                 let binding = rmp_serde::to_vec(receipt).unwrap();
//                 let value = binding.as_slice();
//                 table.insert(&txid.store(), value)?;
//                 let trace = TraceNode {
//                     trace_type: CacheTableIndex::BRC20_EVENTS,
//                     key,
//                 };
//                 Ok(((), trace))
//             })?;
//         Ok(())
//         // self.simulate.save_transaction_receipts(txid, receipt)
//     }
//
//     fn insert_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription: &TransferableLog) -> crate::Result<(), Self::Error> {
//         self.simulate_index
//             .use_write_table(BRC20_TRANSFERABLELOG, |mut table| {
//                 let binding = script_tick_id_key(script, tick, &inscription.inscription_id);
//                 let key = binding.as_str();
//                 let binding = rmp_serde::to_vec(inscription).unwrap();
//                 let value = binding.as_slice();
//                 table.insert(key, value)?;
//                 let trace = TraceNode {
//                     trace_type: CacheTableIndex::BRC20_TRANSFERABLELOG,
//                     key: string_to_bytes(key),
//                 };
//                 Ok(((), trace))
//             })?;
//         Ok(())
//     }
//
//     fn remove_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
//         self.simulate_index.use_write_table(BRC20_TRANSFERABLELOG, |mut table| {
//             let binding = script_tick_id_key(script, tick, inscription_id);
//             let key = binding.as_str();
//             table.remove(key)?;
//             let trace = TraceNode {
//                 trace_type: CacheTableIndex::BRC20_TRANSFERABLELOG,
//                 key: string_to_bytes(key),
//             };
//             Ok(((), trace))
//         })?;
//         Ok(())
//     }
//
//     fn insert_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId, transfer_info: TransferInfo) -> crate::Result<(), Self::Error> {
//         self.simulate_index.use_write_table(BRC20_INSCRIBE_TRANSFER, |mut table| {
//             let key = inscription_id.store();
//             let key_bytes = InscriptionIdValue::as_bytes(&key);
//             let binding = rmp_serde::to_vec(&transfer_info).unwrap();
//             let value = binding.as_slice();
//             table.insert(&key, value)?;
//             let trace = TraceNode {
//                 trace_type: CacheTableIndex::BRC20_INSCRIBE_TRANSFER,
//                 key: key_bytes,
//             };
//             Ok(((), trace))
//         })?;
//         Ok(())
//     }
//
//     fn remove_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
//         self.simulate_index.use_write_table(BRC20_INSCRIBE_TRANSFER, |mut table| {
//             let key = inscription_id.store();
//             let key_bytes = InscriptionIdValue::as_bytes(&key);
//             table.remove(&key)?;
//             let trace = TraceNode {
//                 trace_type: CacheTableIndex::BRC20_INSCRIBE_TRANSFER,
//                 key: key_bytes,
//             };
//             Ok(((), trace))
//         })?;
//         Ok(())
//     }
// }

impl<'a, 'db, 'txn> Brc20Reader for SimulateContext<'a, 'db, 'txn> {
    type Error = anyhow::Error;

    fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
        let simulate = self.simulate.get_balances(script_key)?;
        let mut simulate_balances: HashMap<Tick, Balance> = simulate.into_iter()
            .map(|v| {
                (v.tick.clone(), v.clone())
            }).collect();
        let internal = self.use_internal_table(BRC20_BALANCES, |v| {
            get_balances(&v, script_key)
        })?;
        for node in internal {
            let v = simulate_balances.entry(node.tick.clone()).or_insert(node.clone());
            v.transferable_balance = v.transferable_balance + node.transferable_balance;
            v.overall_balance = v.overall_balance + node.overall_balance;
        }
        let ret = simulate_balances
            .into_iter()
            .map(|(k, v)| {
                v.clone()
            }).collect();
        Ok(ret)
    }

    fn get_balance(&self, script_key: &ScriptKey, tick: &Tick) -> crate::Result<Option<Balance>, Self::Error> {
        let ret = self.simulate.get_balance(script_key, tick)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_BALANCES, |table| {
            get_balance(&table, script_key, tick)
        })
    }

    fn get_token_info(&self, tick: &Tick) -> crate::Result<Option<TokenInfo>, Self::Error> {
        let ret = self.simulate.get_token_info(tick)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_TOKEN, |table| {
            get_token_info(&table, tick)
        })
    }

    fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
        let simulate = self.simulate.get_tokens_info()?;
        let internal = self.use_internal_table(BRC20_TOKEN, |table| {
            get_tokens_info(&table)
        })?;
        //     TODO:merge
        todo!()
    }

    fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
        let simulate = self.simulate.get_transaction_receipts(txid)?;
        let internal = self.use_internal_table(BRC20_EVENTS, |table| {
            get_transaction_receipts(&table, txid)
        })?;
        //     TODO:merge
        todo!()
    }

    fn get_transferable(&self, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>, Self::Error> {
        let simulate = self.simulate.get_transferable(script)?;
        let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
            get_transferable(&table, script)
        })?;
        // TODO: merge
        todo!()
    }


    fn get_transferable_by_tick(&self, script: &ScriptKey, tick: &Tick) -> crate::Result<Vec<TransferableLog>, Self::Error> {
        let simulate = self.simulate.get_transferable_by_tick(script, tick)?;
        let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
            get_transferable_by_tick(&table, script, tick)
        })?;
        // TODO:merge
        todo!()
    }

    fn get_transferable_by_id(&self, script: &ScriptKey, inscription_id: &InscriptionId) -> crate::Result<Option<TransferableLog>, Self::Error> {
        let simulate = self.simulate.get_transferable_by_id(script, inscription_id)?;
        if let Some(ret) = simulate {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
            get_transferable_by_id(&table, script, inscription_id)
        })
    }

    fn get_inscribe_transfer_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<TransferInfo>, Self::Error> {
        let simulate = self.simulate.get_inscribe_transfer_inscription(inscription_id)?;
        if let Some(ret) = simulate {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_INSCRIBE_TRANSFER, |table| {
            get_inscribe_transfer_inscription(&table, inscription_id)
        })
    }
}

impl<'a, 'db, 'txn> Brc20ReaderWriter for SimulateContext<'a, 'db, 'txn> {
    fn update_token_balance(&mut self, script_key: &ScriptKey, new_balance: Balance) -> crate::Result<(), Self::Error> {
        self.simulate.update_token_balance(script_key, new_balance)
    }

    fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> crate::Result<(), Self::Error> {
        self.simulate.insert_token_info(tick, new_info)
    }


    fn update_mint_token_info(&mut self, tick: &Tick, minted_amt: u128, minted_block_number: u32) -> crate::Result<(), Self::Error> {
        self.simulate.update_mint_token_info(tick, minted_amt, minted_block_number)
    }

    fn save_transaction_receipts(&mut self, txid: &Txid, receipt: &[Receipt]) -> crate::Result<(), Self::Error> {
        self.simulate.save_transaction_receipts(txid, receipt)
    }

    fn insert_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription: &TransferableLog) -> crate::Result<(), Self::Error> {
        self.simulate.insert_transferable(script, tick, inscription)
    }

    fn remove_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
        self.simulate.remove_transferable(script, tick, inscription_id)
    }

    fn insert_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId, transfer_info: TransferInfo) -> crate::Result<(), Self::Error> {
        self.simulate.insert_inscribe_transfer_inscription(inscription_id, transfer_info)
    }

    fn remove_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
        self.simulate.remove_inscribe_transfer_inscription(inscription_id)
    }
}

#[test]
pub fn test_asd() {}

#[test]
fn test_write() {
    let tmpfile = NamedTempFile::new().unwrap();
    let db = Database::builder()
        .create(tmpfile.path())
        .unwrap();
    let table_definition: TableDefinition<u64, u64> = TableDefinition::new("x");

    {
        let mut wtx = db.begin_write().unwrap();
        let mut table = wtx.open_table(table_definition).unwrap();
        table.insert(1, 1).unwrap();
        let vv = table.get(&1).unwrap().unwrap().value();
        assert_eq!(vv, 1);
    }


    {
        let mut wtx = db.begin_write().unwrap();
        let mut table = wtx.open_table(table_definition).unwrap();
        table.insert(2, 2).unwrap();
        let vv = table.get(&2).unwrap().unwrap().value();
        assert_eq!(vv, 2);
        let v1 = table.get(&1).unwrap().unwrap().value();
        assert_eq!(v1, 1);
    }

    // wtx.commit().unwrap();
    // {
    //     let rtx = db.begin_read().unwrap();
    //     let table = rtx.open_table(table_definition).unwrap();
    //     assert_eq!(table.get(&1).unwrap().unwrap().value(), 1);
    // }
}

