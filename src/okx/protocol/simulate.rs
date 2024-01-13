// use std::collections::HashMap;
// use std::sync::Arc;
// use bitcoin::{OutPoint, Txid, TxOut};
// use redb::{Database, ReadableTable, ReadOnlyTable, ReadTransaction, RedbKey, RedbValue, Table, TableDefinition, WriteTransaction};
// use tempfile::NamedTempFile;
// use crate::{Index, InscriptionId};
// use crate::index::{BRC20_BALANCES, BRC20_EVENTS, BRC20_INSCRIBE_TRANSFER, BRC20_TOKEN, BRC20_TRANSFERABLELOG, COLLECTIONS_INSCRIPTION_ID_TO_KINDS, COLLECTIONS_KEY_TO_INSCRIPTION_ID, InscriptionIdValue, ORD_TX_TO_OPERATIONS, OUTPOINT_TO_ENTRY, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, TxidValue};
// use crate::index::entry::Entry;
// use crate::okx::datastore::brc20::{Balance, Brc20Reader, Brc20ReaderWriter, Receipt, Tick, TokenInfo, TransferableLog, TransferInfo};
// use crate::okx::datastore::brc20::redb::{script_tick_id_key, script_tick_key};
// use crate::okx::datastore::brc20::redb::table::{get_balance, get_balances, get_inscribe_transfer_inscription, get_token_info, get_tokens_info, get_transaction_receipts, get_transferable, get_transferable_by_id, get_transferable_by_tick};
// use crate::okx::datastore::cache::{CacheTableIndex, CacheWriter};
// use crate::okx::datastore::ScriptKey;
// use crate::okx::protocol::trace::{BalanceDelta, IndexTracer, MintTokenInfoDelta, string_to_bytes, TraceNode};
//
// pub struct SimulateContext {
//     simulate: CacheWriter,
//     internal_index: Arc<Index>,
// }
//
// impl SimulateContext {
//     // pub fn new(
//     //     internal_index: Arc<Index>, simulate: Context) -> crate::Result<Self> {
//     //     // let mut simulate_tx = simulate_index.begin_write()?;
//     //     // let ctx = Context {
//     //     //     chain,
//     //     //     tx_out_cache,
//     //     //     hit: 0,
//     //     //     miss: 0,
//     //     //     ORD_TX_TO_OPERATIONS: &mut simulate_tx.open_table(ORD_TX_TO_OPERATIONS)?,
//     //     //     COLLECTIONS_KEY_TO_INSCRIPTION_ID: &mut simulate_tx.open_table(COLLECTIONS_KEY_TO_INSCRIPTION_ID)?,
//     //     //     COLLECTIONS_INSCRIPTION_ID_TO_KINDS: &mut simulate_tx.open_table(COLLECTIONS_INSCRIPTION_ID_TO_KINDS)?,
//     //     //     SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY: &mut simulate_tx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?,
//     //     //     OUTPOINT_TO_ENTRY: &mut simulate_tx.open_table(OUTPOINT_TO_ENTRY)?,
//     //     //     BRC20_BALANCES: &mut simulate_tx.open_table(BRC20_BALANCES)?,
//     //     //     BRC20_TOKEN: &mut simulate_tx.open_table(BRC20_TOKEN)?,
//     //     //     BRC20_EVENTS: &mut simulate_tx.open_table(BRC20_EVENTS)?,
//     //     //     BRC20_TRANSFERABLELOG: &mut simulate_tx.open_table(BRC20_TRANSFERABLELOG)?,
//     //     //     BRC20_INSCRIBE_TRANSFER: &mut simulate_tx.open_table(BRC20_INSCRIBE_TRANSFER)?,
//     //     // };
//     //     Ok(Self {
//     //         // simulate: ctx,
//     //         internal_index,
//     //         simulate,
//     //     })
//     // }
//     pub fn new(simulate: CacheWriter, internal_index: Arc<Index>) -> Self {
//         Self { simulate, internal_index }
//     }
// }
//
// impl SimulateContext {
//     fn use_internal_table<K: RedbKey + 'static, V: RedbValue + 'static, T>(&self,
//                                                                            table_def: TableDefinition<K, V>,
//                                                                            f: impl FnOnce(ReadOnlyTable<K, V>) -> crate::Result<T>) -> crate::Result<T> {
//         let rtx = self.internal_index.begin_read()?;
//         let table = rtx.0.open_table(table_def)?;
//         let ret = f(table);
//         ret
//     }
// }
//
// // impl Brc20Reader for SimulateContext {
// //     type Error = anyhow::Error;
// //
// //     fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
// //         let simulate = self.simulate_index.use_read_table(BRC20_BALANCES, |table| {
// //             get_balances(&table, script_key)
// //         })?;
// //         let mut simulate_balances: HashMap<Tick, Balance> = simulate.into_iter()
// //             .map(|v| {
// //                 (v.tick.clone(), v.clone())
// //             }).collect();
// //         let internal = self.use_internal_table(BRC20_BALANCES, |v| {
// //             get_balances(&v, script_key)
// //         })?;
// //         for node in internal {
// //             let v = simulate_balances.entry(node.tick.clone()).or_insert(node.clone());
// //             v.transferable_balance = v.transferable_balance + node.transferable_balance;
// //             v.overall_balance = v.overall_balance + node.overall_balance;
// //         }
// //         let ret = simulate_balances
// //             .into_iter()
// //             .map(|(k, v)| {
// //                 v.clone()
// //             }).collect();
// //         Ok(ret)
// //     }
// //
// //     fn get_balance(&self, script_key: &ScriptKey, tick: &Tick) -> crate::Result<Option<Balance>, Self::Error> {
// //         let ret = self.simulate_index.use_read_table(BRC20_BALANCES, |table| {
// //             get_balance(&table, script_key, tick)
// //         })?;
// //         if let Some(ret) = ret {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_BALANCES, |table| {
// //             get_balance(&table, script_key, tick)
// //         })
// //     }
// //
// //     fn get_token_info(&self, tick: &Tick) -> crate::Result<Option<TokenInfo>, Self::Error> {
// //         let ret = self.simulate_index.use_read_table(BRC20_TOKEN, |table| {
// //             get_token_info(&table, tick)
// //         })?;
// //         if let Some(ret) = ret {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_TOKEN, |table| {
// //             get_token_info(&table, tick)
// //         })
// //     }
// //
// //     fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
// //         let simulate = self.simulate_index.use_read_table(BRC20_TOKEN, |table| {
// //             get_tokens_info(&table)
// //         })?;
// //         let internal = self.use_internal_table(BRC20_TOKEN, |table| {
// //             get_tokens_info(&table)
// //         })?;
// //         //     TODO:merge
// //         todo!()
// //     }
// //
// //     fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
// //         let simulate = self.simulate_index.use_read_table(BRC20_EVENTS, |table| {
// //             get_transaction_receipts(&table, txid)
// //         })?;
// //         let internal = self.use_internal_table(BRC20_EVENTS, |table| {
// //             get_transaction_receipts(&table, txid)
// //         })?;
// //         //     TODO:merge
// //         todo!()
// //     }
// //
// //     fn get_transferable(&self, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>, Self::Error> {
// //         let simulate = self.simulate_index.use_read_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable(&table, script)
// //         })?;
// //         let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable(&table, script)
// //         })?;
// //         // TODO: merge
// //         todo!()
// //     }
// //
// //
// //     fn get_transferable_by_tick(&self, script: &ScriptKey, tick: &Tick) -> crate::Result<Vec<TransferableLog>, Self::Error> {
// //         let simulate = self.simulate_index.use_read_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable_by_tick(&table, script, tick)
// //         })?;
// //         let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable_by_tick(&table, script, tick)
// //         })?;
// //         // TODO:merge
// //         todo!()
// //     }
// //
// //     fn get_transferable_by_id(&self, script: &ScriptKey, inscription_id: &InscriptionId) -> crate::Result<Option<TransferableLog>, Self::Error> {
// //         let simulate = self.simulate_index.use_read_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable_by_id(&table, script, inscription_id)
// //         })?;
// //         if let Some(ret) = simulate {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable_by_id(&table, script, inscription_id)
// //         })
// //     }
// //
// //     fn get_inscribe_transfer_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<TransferInfo>, Self::Error> {
// //         let simulate = self.simulate_index.use_read_table(BRC20_INSCRIBE_TRANSFER, |table| {
// //             get_inscribe_transfer_inscription(&table, inscription_id)
// //         })?;
// //         if let Some(ret) = simulate {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_INSCRIBE_TRANSFER, |table| {
// //             get_inscribe_transfer_inscription(&table, inscription_id)
// //         })
// //     }
// // }
// //
// // impl Brc20ReaderWriter for SimulateContext {
// //     fn update_token_balance(&mut self, script_key: &ScriptKey, new_balance: Balance) -> crate::Result<(), Self::Error> {
// //         self.simulate_index
// //             .use_write_table(BRC20_BALANCES, |mut table| {
// //                 let binding = script_tick_key(script_key, &new_balance.tick);
// //                 let key = binding.as_str();
// //                 let binding = rmp_serde::to_vec(&new_balance).unwrap();
// //                 let value = binding.as_slice();
// //                 // let origin = table.get(key)?.map_or(None, |v| {
// //                 //     Some(v.value().to_vec())
// //                 // });
// //                 table.insert(key, value)?;
// //                 // let mut delta = BalanceDelta::default();
// //                 // if let Some(origin) = origin {
// //                 //     let origin: Balance = rmp_serde::from_slice(&origin).unwrap();
// //                 //     delta.origin_transferable_balance_delta = origin.transferable_balance;
// //                 //     delta.origin_overall_balance_delta = origin.overall_balance;
// //                 //     delta.new_overall_balance_delta = new_balance.overall_balance;
// //                 //     delta.new_transferable_balance_delta = new_balance.transferable_balance;
// //                 // } else {
// //                 //     delta.origin_transferable_balance_delta = 0;
// //                 //     delta.origin_overall_balance_delta = 0;
// //                 //     delta.new_overall_balance_delta = new_balance.overall_balance;
// //                 //     delta.new_transferable_balance_delta = new_balance.transferable_balance;
// //                 // }
// //                 // let op = TraceOperation::Update(rmp_serde::to_vec(&delta).unwrap());
// //                 let node = TraceNode { trace_type: CacheTableIndex::BRC20_BALANCES, key: string_to_bytes(key) };
// //                 Ok(((), node))
// //             })?;
// //         Ok(())
// //     }
// //
// //     fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> crate::Result<(), Self::Error> {
// //         self.simulate_index
// //             .use_write_table(BRC20_TOKEN, |mut table| {
// //                 let binding = tick.to_lowercase().hex();
// //                 let key = binding.as_str();
// //                 let binding = rmp_serde::to_vec(new_info).unwrap();
// //                 let value = binding.as_slice();
// //                 table.insert(key, value)?;
// //
// //                 let trace = TraceNode {
// //                     trace_type: CacheTableIndex::BRC20_TOKEN,
// //                     // operation: TraceOperation::Insert,
// //                     key: string_to_bytes(key),
// //                 };
// //                 Ok(((), trace))
// //             })?;
// //         Ok(())
// //     }
// //
// //
// //     fn update_mint_token_info(&mut self, tick: &Tick, minted_amt: u128, minted_block_number: u32) -> crate::Result<(), Self::Error> {
// //         let mut info = self.get_token_info(tick)?.unwrap_or_else(|| panic!("token {} not exist", tick.as_str()));
// //         let origin = info.minted;
// //         info.minted = minted_amt;
// //         info.latest_mint_number = minted_block_number;
// //         self.simulate_index
// //             .use_write_table(BRC20_TOKEN, |mut table| {
// //                 let binding = tick.to_lowercase().hex();
// //                 let key = binding.as_str();
// //                 let binding = rmp_serde::to_vec(&info).unwrap();
// //                 let value = binding.as_slice();
// //                 table.insert(key, value)?;
// //                 // let delta = MintTokenInfoDelta {
// //                 //     origin_minted: origin,
// //                 //     new_minted: info.minted,
// //                 //     new_latest_mint_number: minted_block_number,
// //                 // };
// //                 let trace = TraceNode {
// //                     trace_type: CacheTableIndex::BRC20_TOKEN,
// //                     // operation: TraceOperation::Update(rmp_serde::to_vec(&delta).unwrap()),
// //                     key: string_to_bytes(key),
// //                 };
// //                 Ok(((), trace))
// //             })?;
// //         Ok(())
// //     }
// //
// //     fn save_transaction_receipts(&mut self, txid: &Txid, receipt: &[Receipt]) -> crate::Result<(), Self::Error> {
// //         self.simulate_index
// //             .use_write_table(BRC20_EVENTS, |mut table| {
// //                 let tx_id_value = txid.store();
// //                 let key = tx_id_value.to_vec();
// //                 let binding = rmp_serde::to_vec(receipt).unwrap();
// //                 let value = binding.as_slice();
// //                 table.insert(&txid.store(), value)?;
// //                 let trace = TraceNode {
// //                     trace_type: CacheTableIndex::BRC20_EVENTS,
// //                     key,
// //                 };
// //                 Ok(((), trace))
// //             })?;
// //         Ok(())
// //         // self.simulate.save_transaction_receipts(txid, receipt)
// //     }
// //
// //     fn insert_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription: &TransferableLog) -> crate::Result<(), Self::Error> {
// //         self.simulate_index
// //             .use_write_table(BRC20_TRANSFERABLELOG, |mut table| {
// //                 let binding = script_tick_id_key(script, tick, &inscription.inscription_id);
// //                 let key = binding.as_str();
// //                 let binding = rmp_serde::to_vec(inscription).unwrap();
// //                 let value = binding.as_slice();
// //                 table.insert(key, value)?;
// //                 let trace = TraceNode {
// //                     trace_type: CacheTableIndex::BRC20_TRANSFERABLELOG,
// //                     key: string_to_bytes(key),
// //                 };
// //                 Ok(((), trace))
// //             })?;
// //         Ok(())
// //     }
// //
// //     fn remove_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
// //         self.simulate_index.use_write_table(BRC20_TRANSFERABLELOG, |mut table| {
// //             let binding = script_tick_id_key(script, tick, inscription_id);
// //             let key = binding.as_str();
// //             table.remove(key)?;
// //             let trace = TraceNode {
// //                 trace_type: CacheTableIndex::BRC20_TRANSFERABLELOG,
// //                 key: string_to_bytes(key),
// //             };
// //             Ok(((), trace))
// //         })?;
// //         Ok(())
// //     }
// //
// //     fn insert_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId, transfer_info: TransferInfo) -> crate::Result<(), Self::Error> {
// //         self.simulate_index.use_write_table(BRC20_INSCRIBE_TRANSFER, |mut table| {
// //             let key = inscription_id.store();
// //             let key_bytes = InscriptionIdValue::as_bytes(&key);
// //             let binding = rmp_serde::to_vec(&transfer_info).unwrap();
// //             let value = binding.as_slice();
// //             table.insert(&key, value)?;
// //             let trace = TraceNode {
// //                 trace_type: CacheTableIndex::BRC20_INSCRIBE_TRANSFER,
// //                 key: key_bytes,
// //             };
// //             Ok(((), trace))
// //         })?;
// //         Ok(())
// //     }
// //
// //     fn remove_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
// //         self.simulate_index.use_write_table(BRC20_INSCRIBE_TRANSFER, |mut table| {
// //             let key = inscription_id.store();
// //             let key_bytes = InscriptionIdValue::as_bytes(&key);
// //             table.remove(&key)?;
// //             let trace = TraceNode {
// //                 trace_type: CacheTableIndex::BRC20_INSCRIBE_TRANSFER,
// //                 key: key_bytes,
// //             };
// //             Ok(((), trace))
// //         })?;
// //         Ok(())
// //     }
// // }
//
// // impl Brc20Reader for SimulateContext {
// //     type Error = anyhow::Error;
// //
// //     fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
// //         let simulate = self.simulate.get_balances(script_key)?;
// //         let mut simulate_balances: HashMap<Tick, Balance> = simulate.into_iter()
// //             .map(|v| {
// //                 (v.tick.clone(), v.clone())
// //             }).collect();
// //         let internal = self.use_internal_table(BRC20_BALANCES, |v| {
// //             get_balances(&v, script_key)
// //         })?;
// //         for node in internal {
// //             let v = simulate_balances.entry(node.tick.clone()).or_insert(node.clone());
// //             v.transferable_balance = v.transferable_balance + node.transferable_balance;
// //             v.overall_balance = v.overall_balance + node.overall_balance;
// //         }
// //         let ret = simulate_balances
// //             .into_iter()
// //             .map(|(k, v)| {
// //                 v.clone()
// //             }).collect();
// //         Ok(ret)
// //     }
// //
// //     fn get_balance(&self, script_key: &ScriptKey, tick: &Tick) -> crate::Result<Option<Balance>, Self::Error> {
// //         let ret = self.simulate.get_balance(script_key, tick)?;
// //         if let Some(ret) = ret {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_BALANCES, |table| {
// //             get_balance(&table, script_key, tick)
// //         })
// //     }
// //
// //     fn get_token_info(&self, tick: &Tick) -> crate::Result<Option<TokenInfo>, Self::Error> {
// //         let ret = self.simulate.get_token_info(tick)?;
// //         if let Some(ret) = ret {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_TOKEN, |table| {
// //             get_token_info(&table, tick)
// //         })
// //     }
// //
// //     fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
// //         let simulate = self.simulate.get_tokens_info()?;
// //         let internal = self.use_internal_table(BRC20_TOKEN, |table| {
// //             get_tokens_info(&table)
// //         })?;
// //         //     TODO:merge
// //         todo!()
// //     }
// //
// //     fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
// //         let simulate = self.simulate.get_transaction_receipts(txid)?;
// //         let internal = self.use_internal_table(BRC20_EVENTS, |table| {
// //             get_transaction_receipts(&table, txid)
// //         })?;
// //         //     TODO:merge
// //         todo!()
// //     }
// //
// //     fn get_transferable(&self, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>, Self::Error> {
// //         let simulate = self.simulate.get_transferable(script)?;
// //         let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable(&table, script)
// //         })?;
// //         // TODO: merge
// //         todo!()
// //     }
// //
// //
// //     fn get_transferable_by_tick(&self, script: &ScriptKey, tick: &Tick) -> crate::Result<Vec<TransferableLog>, Self::Error> {
// //         let simulate = self.simulate.get_transferable_by_tick(script, tick)?;
// //         let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable_by_tick(&table, script, tick)
// //         })?;
// //         // TODO:merge
// //         todo!()
// //     }
// //
// //     fn get_transferable_by_id(&self, script: &ScriptKey, inscription_id: &InscriptionId) -> crate::Result<Option<TransferableLog>, Self::Error> {
// //         let simulate = self.simulate.get_transferable_by_id(script, inscription_id)?;
// //         if let Some(ret) = simulate {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
// //             get_transferable_by_id(&table, script, inscription_id)
// //         })
// //     }
// //
// //     fn get_inscribe_transfer_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<TransferInfo>, Self::Error> {
// //         let simulate = self.simulate.get_inscribe_transfer_inscription(inscription_id)?;
// //         if let Some(ret) = simulate {
// //             return Ok(Some(ret));
// //         }
// //         self.use_internal_table(BRC20_INSCRIBE_TRANSFER, |table| {
// //             get_inscribe_transfer_inscription(&table, inscription_id)
// //         })
// //     }
// // }
// //
// // impl Brc20ReaderWriter for SimulateContext {
// //     fn update_token_balance(&mut self, script_key: &ScriptKey, new_balance: Balance) -> crate::Result<(), Self::Error> {
// //         self.simulate.update_token_balance(script_key, new_balance)
// //     }
// //
// //     fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> crate::Result<(), Self::Error> {
// //         self.simulate.insert_token_info(tick, new_info)
// //     }
// //
// //
// //     fn update_mint_token_info(&mut self, tick: &Tick, minted_amt: u128, minted_block_number: u32) -> crate::Result<(), Self::Error> {
// //         self.simulate.update_mint_token_info(tick, minted_amt, minted_block_number)
// //     }
// //
// //     fn save_transaction_receipts(&mut self, txid: &Txid, receipt: &[Receipt]) -> crate::Result<(), Self::Error> {
// //         self.simulate.save_transaction_receipts(txid, receipt)
// //     }
// //
// //     fn insert_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription: &TransferableLog) -> crate::Result<(), Self::Error> {
// //         self.simulate.insert_transferable(script, tick, inscription)
// //     }
// //
// //     fn remove_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
// //         self.simulate.remove_transferable(script, tick, inscription_id)
// //     }
// //
// //     fn insert_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId, transfer_info: TransferInfo) -> crate::Result<(), Self::Error> {
// //         self.simulate.insert_inscribe_transfer_inscription(inscription_id, transfer_info)
// //     }
// //
// //     fn remove_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
// //         self.simulate.remove_inscribe_transfer_inscription(inscription_id)
// //     }
// // }
//
// impl Brc20Reader for SimulateContext {
//     type Error = anyhow::Error;
//
//     fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
//         // self.simulate.use_cache_mut(CacheTableIndex::BRC20_BALANCES,|table|{
//         //
//         // })
//         // let simulate = self.simulate.get_balances(script_key)?;
//         // let mut simulate_balances: HashMap<Tick, Balance> = simulate.into_iter()
//         //     .map(|v| {
//         //         (v.tick.clone(), v.clone())
//         //     }).collect();
//         // let internal = self.use_internal_table(BRC20_BALANCES, |v| {
//         //     get_balances(&v, script_key)
//         // })?;
//         // for node in internal {
//         //     let v = simulate_balances.entry(node.tick.clone()).or_insert(node.clone());
//         //     v.transferable_balance = v.transferable_balance + node.transferable_balance;
//         //     v.overall_balance = v.overall_balance + node.overall_balance;
//         // }
//         // let ret = simulate_balances
//         //     .into_iter()
//         //     .map(|(k, v)| {
//         //         v.clone()
//         //     }).collect();
//         // Ok(ret)
//         todo!()
//     }
//
//     fn get_balance(&self, script_key: &ScriptKey, tick: &Tick) -> crate::Result<Option<Balance>, Self::Error> {
//         let ret = self.simulate.use_cache(CacheTableIndex::BRC20_BALANCES, |table| {
//             if table.is_none() {
//                 return Ok::<Option<Balance>, Self::Error>(None);
//             }
//             let key = script_tick_key(script_key, tick);
//             let key = key.as_str();
//             let table = table.unwrap();
//             let value = table.get(string_to_bytes(key));
//             if value.is_none() {
//                 return Ok::<Option<Balance>, Self::Error>(None);
//             }
//             let bal: Balance = rmp_serde::from_slice(&value.unwrap()).unwrap();
//             Ok(Some(bal))
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
//         let ret = self.simulate
//             .use_cache(CacheTableIndex::BRC20_TOKEN, |table| {
//                 if table.is_none() {
//                     return Ok::<Option<TokenInfo>, Self::Error>(None);
//                 }
//                 let table = table.unwrap();
//                 let binding = tick.to_lowercase().hex();
//                 let key = binding.as_str();
//                 let value = table.get(string_to_bytes(key));
//                 if value.is_none() {
//                     return Ok::<Option<TokenInfo>, Self::Error>(None);
//                 }
//                 let value = value.unwrap();
//                 let info: TokenInfo = rmp_serde::from_slice(&value).unwrap();
//                 return Ok(Some(info));
//             })?;
//         if let Some(ret) = ret {
//             return Ok(Some(ret));
//         }
//         self.use_internal_table(BRC20_TOKEN, |table| {
//             get_token_info(&table, tick)
//         })
//     }
//
//     fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
//         // let simulate = self.simulate.get_tokens_info()?;
//         // let internal = self.use_internal_table(BRC20_TOKEN, |table| {
//         //     get_tokens_info(&table)
//         // })?;
//         //     TODO:merge
//         todo!()
//     }
//
//     fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
//         let simulate = self.simulate
//             .use_cache(CacheTableIndex::BRC20_EVENTS, |table| {
//                 if table.is_none() {
//                     return Ok::<Option<Vec<Receipt>>, Self::Error>(None);
//                 }
//                 let table = table.unwrap();
//                 let key = &txid.store();
//                 let key: Vec<u8> = key.to_vec();
//                 // let key = TxidValue::as_bytes(&key).as_ref();
//                 let value = table.get(key);
//                 if value.is_none() {
//                     return Ok::<Option<Vec<Receipt>>, Self::Error>(None);
//                 }
//                 let value = value.unwrap();
//                 let ret = rmp_serde::from_slice::<Vec<Receipt>>(&value).unwrap();
//                 Ok(Some(ret))
//             })?;
//         let internal = self.use_internal_table(BRC20_EVENTS, |table| {
//             get_transaction_receipts(&table, txid)
//         })?;
//         //     TODO:merge
//         todo!()
//     }
//
//     fn get_transferable(&self, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>, Self::Error> {
//         // let simulate = self.simulate.get_transferable(script)?;
//         // let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
//         //     get_transferable(&table, script)
//         // })?;
//         // TODO: merge
//         todo!()
//     }
//
//
//     fn get_transferable_by_tick(&self, script: &ScriptKey, tick: &Tick) -> crate::Result<Vec<TransferableLog>, Self::Error> {
//         // let simulate = self.simulate.get_transferable_by_tick(script, tick)?;
//         // let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
//         //     get_transferable_by_tick(&table, script, tick)
//         // })?;
//         // TODO:merge
//         todo!()
//     }
//
//     fn get_transferable_by_id(&self, script: &ScriptKey, inscription_id: &InscriptionId) -> crate::Result<Option<TransferableLog>, Self::Error> {
//         // let simulate = self.simulate.get_transferable_by_id(script, inscription_id)?;
//         // if let Some(ret) = simulate {
//         //     return Ok(Some(ret));
//         // }
//         // self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
//         //     get_transferable_by_id(&table, script, inscription_id)
//         // })
//         todo!()
//     }
//
//     fn get_inscribe_transfer_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<TransferInfo>, Self::Error> {
//         let simulate = self.simulate
//             .use_cache(CacheTableIndex::BRC20_INSCRIBE_TRANSFER, |table| {
//                 if table.is_none() {
//                     return Ok::<Option<TransferInfo>, Self::Error>(None);
//                 }
//                 let table = table.unwrap();
//                 let key = inscription_id.store();
//                 let key_bytes = InscriptionIdValue::as_bytes(&key);
//                 let value = table.get(key_bytes);
//                 if value.is_none() {
//                     return Ok::<Option<TransferInfo>, Self::Error>(None);
//                 }
//                 let value = value.unwrap();
//                 let ret: TransferInfo = rmp_serde::from_slice::<TransferInfo>(&value).unwrap();
//                 return Ok(Some(ret));
//             })?;
//         // let simulate = self.simulate.get_inscribe_transfer_inscription(inscription_id)?;
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
//         self.simulate.use_cache_mut(CacheTableIndex::BRC20_BALANCES, |table| {
//             let key = script_tick_key(script_key, &new_balance.tick);
//             let key = key.as_str();
//             let binding = rmp_serde::to_vec(&new_balance).unwrap();
//             let value = binding.as_slice();
//             table.insert(string_to_bytes(key), value.to_vec());
//             Ok(())
//         })
//     }
//
//     fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> crate::Result<(), Self::Error> {
//         self.simulate.use_cache_mut(CacheTableIndex::BRC20_TOKEN, |table| {
//             let binding = tick.to_lowercase().hex();
//             let key = binding.as_str();
//             let binding = rmp_serde::to_vec(new_info).unwrap();
//             let value = binding.as_slice();
//             table.insert(string_to_bytes(key), value.to_vec());
//             Ok(())
//         })
//     }
//
//
//     fn update_mint_token_info(&mut self, tick: &Tick, minted_amt: u128, minted_block_number: u32) -> crate::Result<(), Self::Error> {
//         let mut info = self.
//             get_token_info(tick)?.unwrap_or_else(|| panic!("token {} not exist", tick.as_str()));
//         info.minted = minted_amt;
//         info.latest_mint_number = minted_block_number;
//         self.simulate
//             .use_cache_mut(CacheTableIndex::BRC20_TOKEN, |table| {
//                 let binding = tick.to_lowercase().hex();
//                 let key = binding.as_str();
//                 let value = rmp_serde::to_vec(&info).unwrap();
//                 table.insert(string_to_bytes(key), value.to_vec());
//                 Ok(())
//             })
//     }
//
//     fn save_transaction_receipts(&mut self, txid: &Txid, receipts: &[Receipt]) -> crate::Result<(), Self::Error> {
//         self.simulate.use_cache_mut(CacheTableIndex::BRC20_EVENTS, |table| {
//             let v = txid.store();
//             let key = v.to_vec();
//             let value = rmp_serde::to_vec(receipts).unwrap();
//             table.insert(key, value);
//             Ok(())
//         })
//     }
//
//     fn insert_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription: &TransferableLog) -> crate::Result<(), Self::Error> {
//         self.simulate
//             .use_cache_mut(CacheTableIndex::BRC20_TRANSFERABLELOG, |table| {
//                 let binding = script_tick_id_key(script, tick, &inscription.inscription_id);
//                 let key = binding.as_str();
//                 let binding = rmp_serde::to_vec(inscription).unwrap();
//                 let value = binding.as_slice();
//                 table.insert(string_to_bytes(key), value.to_vec());
//                 Ok(())
//             })
//     }
//
//     fn remove_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
//         self.simulate.use_cache_mut(CacheTableIndex::BRC20_TRANSFERABLELOG, |table| {
//             let binding = script_tick_id_key(script, tick, inscription_id);
//             let key = binding.as_str();
//             table.remove(string_to_bytes(key));
//             Ok(())
//         })
//     }
//
//     fn insert_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId, transfer_info: TransferInfo) -> crate::Result<(), Self::Error> {
//         self.simulate.use_cache_mut(CacheTableIndex::BRC20_INSCRIBE_TRANSFER, |table| {
//             let key = inscription_id.store();
//             let key_bytes = InscriptionIdValue::as_bytes(&key);
//             let value = rmp_serde::to_vec(&transfer_info).unwrap();
//             table.insert(key_bytes, value);
//             Ok(())
//         })
//     }
//
//     fn remove_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
//         self.simulate.use_cache_mut(CacheTableIndex::BRC20_INSCRIBE_TRANSFER, |table| {
//             let key = inscription_id.store();
//             let key_bytes = InscriptionIdValue::as_bytes(&key);
//             table.remove(key_bytes);
//             Ok(())
//         })
//     }
// }
//
// #[test]
// pub fn test_asd() {}
//
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


//////////

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
use anyhow::anyhow;
use bitcoin::{Network, Txid};
use redb::{Database, MultimapTable, ReadableTable, ReadOnlyTable, RedbKey, RedbValue, Table, TableDefinition, WriteTransaction};
use tempfile::NamedTempFile;
use crate::{Index, InscriptionId, SatPoint};
use crate::index::{BRC20_BALANCES, BRC20_EVENTS, BRC20_INSCRIBE_TRANSFER, BRC20_TOKEN, BRC20_TRANSFERABLELOG, COLLECTIONS_INSCRIPTION_ID_TO_KINDS, COLLECTIONS_KEY_TO_INSCRIPTION_ID, InscriptionEntryValue, InscriptionIdValue, OUTPOINT_TO_ENTRY, OutPointValue, SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, TxidValue};
use crate::index::simulator::processor::IndexWrapper;
use crate::okx::datastore::brc20::{Balance, Brc20Reader, Brc20ReaderWriter, Receipt, Tick, TokenInfo, TransferableLog, TransferInfo};
use crate::okx::datastore::brc20::redb::table::{get_balance, get_balances, get_inscribe_transfer_inscription, get_token_info, get_tokens_info, get_transaction_receipts, get_transferable, get_transferable_by_id, get_transferable_by_tick, insert_inscribe_transfer_inscription, insert_token_info, insert_transferable, remove_inscribe_transfer_inscription, remove_transferable, save_transaction_receipts, update_token_balance};
use crate::okx::datastore::ord::{InscriptionOp, OrdReader, OrdReaderWriter};
use crate::okx::datastore::ord::collections::CollectionKind;
use crate::okx::datastore::ord::redb::table::{get_collection_inscription_id, get_collections_of_inscription, get_inscription_number_by_sequence_number, get_transaction_operations, get_txout_by_outpoint, save_transaction_operations, set_inscription_attributes, set_inscription_by_collection_key};
use crate::okx::datastore::ScriptKey;
use crate::okx::protocol::ContextTrait;

#[derive(Clone)]
pub struct SimulateContext<'a, 'db, 'txn> {
    pub network: Network,
    pub current_height: u32,
    pub current_block_time: u32,
    pub internal_index: IndexWrapper,
    pub(crate) ORD_TX_TO_OPERATIONS: Rc<RefCell<Table<'db, 'txn, &'static TxidValue, &'static [u8]>>>,
    pub(crate) COLLECTIONS_KEY_TO_INSCRIPTION_ID:
    Rc<RefCell<Table<'db, 'txn, &'static str, InscriptionIdValue>>>,
    pub(crate) COLLECTIONS_INSCRIPTION_ID_TO_KINDS:
    Rc<RefCell<Table<'db, 'txn, InscriptionIdValue, &'static [u8]>>>,
    pub(crate) SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY:
    Rc<RefCell<Table<'db, 'txn, u32, InscriptionEntryValue>>>,
    pub(crate) OUTPOINT_TO_ENTRY: Rc<RefCell<Table<'db, 'txn, &'static OutPointValue, &'static [u8]>>>,

    // BRC20 tables
    pub(crate) BRC20_BALANCES: Rc<RefCell<Table<'db, 'txn, &'static str, &'static [u8]>>>,
    pub(crate) BRC20_TOKEN: Rc<RefCell<Table<'db, 'txn, &'static str, &'static [u8]>>>,
    pub(crate) BRC20_EVENTS: Rc<RefCell<Table<'db, 'txn, &'static TxidValue, &'static [u8]>>>,
    pub(crate) BRC20_TRANSFERABLELOG: Rc<RefCell<Table<'db, 'txn, &'static str, &'static [u8]>>>,
    pub(crate) BRC20_INSCRIBE_TRANSFER: Rc<RefCell<Table<'db, 'txn, InscriptionIdValue, &'static [u8]>>>,
    pub _marker_a: PhantomData<&'a ()>,
}

impl<'a, 'db, 'txn> Brc20Reader for SimulateContext<'a, 'db, 'txn> {
    type Error = anyhow::Error;

    fn get_balances(&self, script_key: &ScriptKey) -> crate::Result<Vec<Balance>, Self::Error> {
        let balances = self.BRC20_BALANCES.borrow();
        let table = balances.deref();
        let simulate = get_balances(table, script_key)?;
        let internal = self.use_internal_table(BRC20_BALANCES, |v| {
            get_balances(&v, script_key)
        })?;
        let mut simulate_balances: HashMap<Tick, Balance> = simulate.into_iter()
            .map(|v| {
                (v.tick.clone(), v.clone())
            }).collect();
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
        let table = self.BRC20_BALANCES.borrow();
        let table = table.deref();
        let ret = get_balance(table, script_key, tick)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_BALANCES, |table| {
            get_balance(&table, script_key, tick)
        })
    }

    fn get_token_info(&self, tick: &Tick) -> crate::Result<Option<TokenInfo>, Self::Error> {
        let table = self.BRC20_TOKEN.borrow();
        let table = table.deref();
        let ret = get_token_info(table, tick)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_TOKEN, |table| {
            get_token_info(&table, tick)
        })
    }

    fn get_tokens_info(&self) -> crate::Result<Vec<TokenInfo>, Self::Error> {
        let binding = self.BRC20_TOKEN.borrow();
        let table = binding.deref();
        let ret = get_tokens_info(table)?;
        let mut token_map = ret.into_iter().map(|v| {
            (v.tick.clone(), v)
        }).collect::<HashMap<Tick, TokenInfo>>();
        let internal = self.use_internal_table(BRC20_TOKEN, |table| {
            get_tokens_info(&table)
        })?;
        for node in internal {
            if !token_map.contains_key(&node.tick) {
                token_map.insert(node.tick.clone(), node.clone());
            }
        }
        let ret = token_map.into_iter().map(|(_, v)| {
            v
        }).collect();
        Ok(ret)
    }

    fn get_transaction_receipts(&self, txid: &Txid) -> crate::Result<Vec<Receipt>, Self::Error> {
        let binding = self.BRC20_EVENTS.borrow();
        let table = binding.deref();
        let ret = get_transaction_receipts(table, txid)?;
        let mut simulate_receipts = ret.into_iter().map(|v| {
            (v.inscription_id.clone(), v)
        }).collect::<HashMap<InscriptionId, Receipt>>();
        let internal = self.use_internal_table(BRC20_EVENTS, |table| {
            get_transaction_receipts(&table, txid)
        })?;
        for node in internal {
            if !simulate_receipts.contains_key(&node.inscription_id) {
                simulate_receipts.insert(node.inscription_id.clone(), node.clone());
            }
        }
        let ret = simulate_receipts.into_iter().map(|(_, v)| {
            v
        }).collect();
        Ok(ret)
    }

    fn get_transferable(&self, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>, Self::Error> {
        let binding = self.BRC20_TRANSFERABLELOG.borrow();
        let table = binding.deref();
        let ret = get_transferable(table, script)?;
        let mut simulate_transferable = ret.into_iter().map(|v| {
            (v.inscription_id.clone(), v)
        }).collect::<HashMap<InscriptionId, TransferableLog>>();
        let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
            get_transferable(&table, script)
        })?;
        for node in internal {
            if !simulate_transferable.contains_key(&node.inscription_id) {
                simulate_transferable.insert(node.inscription_id.clone(), node.clone());
            }
        }
        let ret = simulate_transferable.into_iter().map(|(_, v)| {
            v
        }).collect();
        Ok(ret)
    }

    fn get_transferable_by_tick(&self, script: &ScriptKey, tick: &Tick) -> crate::Result<Vec<TransferableLog>, Self::Error> {
        let binding = self.BRC20_TRANSFERABLELOG.borrow();
        let table = binding.deref();
        let ret = get_transferable_by_tick(table, script, tick)?;
        let mut simulate_transferable = ret.into_iter().map(|v| {
            (v.inscription_id.clone(), v)
        }).collect::<HashMap<InscriptionId, TransferableLog>>();
        let internal = self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
            get_transferable_by_tick(&table, script, tick)
        })?;
        for node in internal {
            if !simulate_transferable.contains_key(&node.inscription_id) {
                simulate_transferable.insert(node.inscription_id.clone(), node.clone());
            }
        }
        let ret = simulate_transferable.into_iter().map(|(_, v)| {
            v
        }).collect();
        Ok(ret)
    }

    fn get_transferable_by_id(&self, script: &ScriptKey, inscription_id: &InscriptionId) -> crate::Result<Option<TransferableLog>, Self::Error> {
        let binding = self.BRC20_TRANSFERABLELOG.borrow();
        let table = binding.deref();
        let ret = get_transferable_by_id(table, script, inscription_id)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_TRANSFERABLELOG, |table| {
            get_transferable_by_id(&table, script, inscription_id)
        })
    }

    fn get_inscribe_transfer_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<TransferInfo>, Self::Error> {
        let binding = self.BRC20_INSCRIBE_TRANSFER.borrow();
        let table = binding.deref();
        let ret = get_inscribe_transfer_inscription(table, inscription_id)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        self.use_internal_table(BRC20_INSCRIBE_TRANSFER, |table| {
            get_inscribe_transfer_inscription(&table, inscription_id)
        })
    }
}

impl<'a, 'db, 'txn> Brc20ReaderWriter for SimulateContext<'a, 'db, 'txn> {
    fn update_token_balance(&mut self, script_key: &ScriptKey, new_balance: Balance) -> crate::Result<(), Self::Error> {
        let mut table = self.BRC20_BALANCES.borrow_mut();
        update_token_balance(&mut table, script_key, new_balance)
    }

    fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> crate::Result<(), Self::Error> {
        let mut binding = self.BRC20_TOKEN.borrow_mut();
        let mut table = binding.deref_mut();
        insert_token_info(table, tick, new_info)
    }

    fn update_mint_token_info(&mut self, tick: &Tick, minted_amt: u128, minted_block_number: u32) -> crate::Result<(), Self::Error> {
        let mut binding = self.BRC20_TOKEN.borrow_mut();
        let mut table = binding.deref_mut();
        let mut info = get_token_info(table, tick)?.unwrap_or_else(|| panic!("token {} not exist", tick.as_str()));
        info.minted = minted_amt;
        info.latest_mint_number = minted_block_number;
        insert_token_info(table, tick, &info)
    }

    fn save_transaction_receipts(&mut self, txid: &Txid, receipt: &[Receipt]) -> crate::Result<(), Self::Error> {
        let mut table = self.BRC20_EVENTS.borrow_mut();
        save_transaction_receipts(&mut table, txid, receipt)
    }

    fn insert_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription: &TransferableLog) -> crate::Result<(), Self::Error> {
        let mut table = self.BRC20_TRANSFERABLELOG.borrow_mut();
        insert_transferable(&mut table, script, tick, inscription)
    }

    fn remove_transferable(&mut self, script: &ScriptKey, tick: &Tick, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
        let mut table = self.BRC20_TRANSFERABLELOG.borrow_mut();
        remove_transferable(&mut table, script, tick, inscription_id)
    }

    fn insert_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId, transfer_info: TransferInfo) -> crate::Result<(), Self::Error> {
        let mut table = self.BRC20_INSCRIBE_TRANSFER.borrow_mut();
        insert_inscribe_transfer_inscription(&mut table, inscription_id, transfer_info)
    }

    fn remove_inscribe_transfer_inscription(&mut self, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
        let mut table = self.BRC20_INSCRIBE_TRANSFER.borrow_mut();
        remove_inscribe_transfer_inscription(&mut table, inscription_id)
    }
}

impl<'a, 'db, 'txn> OrdReaderWriter for SimulateContext<'a, 'db, 'txn> {
    fn save_transaction_operations(&mut self, txid: &Txid, operations: &[InscriptionOp]) -> crate::Result<(), Self::Error> {
        let mut table = self.ORD_TX_TO_OPERATIONS.borrow_mut();
        save_transaction_operations(&mut table, txid, operations)
    }

    fn set_inscription_by_collection_key(&mut self, key: &str, inscription_id: &InscriptionId) -> crate::Result<(), Self::Error> {
        let mut table = self.COLLECTIONS_KEY_TO_INSCRIPTION_ID.borrow_mut();
        set_inscription_by_collection_key(&mut table, key, inscription_id)
    }

    fn set_inscription_attributes(&mut self, inscription_id: &InscriptionId, kind: &[CollectionKind]) -> crate::Result<(), Self::Error> {
        let mut table = self.COLLECTIONS_INSCRIPTION_ID_TO_KINDS.borrow_mut();
        set_inscription_attributes(
            &mut table,
            inscription_id,
            kind,
        )
    }
}

impl<'a, 'db, 'txn> OrdReader for SimulateContext<'a, 'db, 'txn> {
    type Error = anyhow::Error;

    fn get_inscription_number_by_sequence_number(&self, sequence_number: u32) -> crate::Result<i32, Self::Error> {
        let binding = self.SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY.borrow();
        let table = binding.deref();
        let ret = get_inscription_number_by_sequence_number(
            table,
            sequence_number,
        )
            .map_err(|e| anyhow!("failed to get inscription number from state! error: {e}"))?;
        if let Some(ret) = ret {
            return Ok(ret);
        }
        self.use_internal_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, |table| {
            get_inscription_number_by_sequence_number(
                &table,
                sequence_number,
            )
        }).map_err(|e| anyhow!("failed to get inscription number from state! error: {e}"))?.ok_or(anyhow!(
      "failed to get inscription number! error: sequence number {} not found",
      sequence_number
    ))
    }

    fn get_script_key_on_satpoint(&mut self, satpoint: &SatPoint, network: Network) -> crate::Result<ScriptKey, Self::Error> {
        let binding = self.OUTPOINT_TO_ENTRY.borrow();
        let table = binding.deref();
        if let Some(tx_out) = get_txout_by_outpoint(table, &satpoint.outpoint)?
        {
            return Ok(ScriptKey::from_script(&tx_out.script_pubkey, network));
        } else {
            let ret = self.use_internal_table(OUTPOINT_TO_ENTRY, |table| {
                get_txout_by_outpoint(&table, &satpoint.outpoint)
            })?;
            if let Some(ret) = ret {
                return Ok(ScriptKey::from_script(&ret.script_pubkey, network));
            }
        }
        return Err(anyhow!("failed to get tx out! error: outpoint {} not found",
        &satpoint.outpoint));
    }

    fn get_transaction_operations(&self, txid: &Txid) -> crate::Result<Vec<InscriptionOp>, Self::Error> {
        let binding = self.ORD_TX_TO_OPERATIONS.borrow();
        let table = binding.deref();
        let simulate = get_transaction_operations(table, txid)?;
        let mut simulate_operations: HashMap<InscriptionId, InscriptionOp> = simulate.into_iter()
            .map(|v| {
                (v.inscription_id.clone(), v.clone())
            }).collect();
        let internal = self.use_internal_table(BRC20_EVENTS, |table| {
            get_transaction_operations(&table, txid)
        })?;
        for node in internal {
            if simulate_operations.contains_key(&node.inscription_id) {
                continue;
            }
            simulate_operations.insert(node.inscription_id.clone(), node.clone());
        }
        let ret = simulate_operations.into_iter().map(|(_, v)| {
            v
        }).collect();
        Ok(ret)
    }

    fn get_collections_of_inscription(&self, inscription_id: &InscriptionId) -> crate::Result<Option<Vec<CollectionKind>>, Self::Error> {
        let binding = self.COLLECTIONS_INSCRIPTION_ID_TO_KINDS.borrow();
        let table = binding.deref();
        let simulate = get_collections_of_inscription(table, inscription_id)?;
        let mut simulate = if let Some(ret) = simulate {
            ret
        } else {
            vec![]
        };

        let internal = self.use_internal_table(COLLECTIONS_INSCRIPTION_ID_TO_KINDS, |table| {
            get_collections_of_inscription(&table, inscription_id)
        })?;
        if let Some(internal) = internal {
            simulate.extend_from_slice(&internal);
        }
        if simulate.is_empty() {
            return Ok(None);
        }
        return Ok(Some(simulate));
    }

    fn get_collection_inscription_id(&self, collection_key: &str) -> crate::Result<Option<InscriptionId>, Self::Error> {
        let binding = self.COLLECTIONS_KEY_TO_INSCRIPTION_ID.borrow();
        let table = binding.deref();
        let ret = get_collection_inscription_id(table, collection_key)?;
        if let Some(ret) = ret {
            return Ok(Some(ret));
        }
        self.use_internal_table(COLLECTIONS_KEY_TO_INSCRIPTION_ID, |table| {
            get_collection_inscription_id(&table, collection_key)
        })
    }
}

impl<'a, 'db, 'txn> ContextTrait for SimulateContext<'a, 'db, 'txn> {
    fn block_height(&self) -> u32 {
        self.current_height
    }

    fn network(&self) -> Network {
        self.network.clone()
    }

    fn block_time(&self) -> u32 {
        self.current_block_time
    }
}

impl<'a, 'db, 'txn> SimulateContext<'a, 'db, 'txn> {
    fn use_internal_table<K: RedbKey + 'static, V: RedbValue + 'static, T>(&self,
                                                                           table_def: TableDefinition<K, V>,
                                                                           f: impl FnOnce(ReadOnlyTable<K, V>) -> crate::Result<T>) -> crate::Result<T> {
        let rtx = self.internal_index.internal.begin_read()?;
        let table = rtx.0.open_table(table_def)?;
        let ret = f(table);
        ret
    }
}